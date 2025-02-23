use crate::{language::Visibility, Engines, Ident};

use super::{
    module::Module, root::Root, submodule_namespace::SubmoduleNamespace, ModulePath, ModulePathBuf,
};

use sway_error::handler::Handler;
use sway_types::span::Span;

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug, Default)]
pub struct Namespace {
    /// An immutable namespace that consists of the names that should always be present, no matter
    /// what module or scope we are currently checking.
    ///
    /// These include external library dependencies and (when it's added) the `std` prelude.
    ///
    /// This is passed through type-checking in order to initialise the namespace of each submodule
    /// within the project.
    init: Module,
    /// The `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `init` namespace before type-checking begins.
    pub(crate) root: Root,
    /// An absolute path from the `root` that represents the current module being checked.
    ///
    /// E.g. when type-checking the root module, this is equal to `[]`. When type-checking a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub(crate) mod_path: ModulePathBuf,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            init: Module::default(),
            mod_path: vec![],
            root: Root::default(),
        }
    }

    pub fn program_id(&self, engines: &Engines) -> &Module {
        self.root
            .module
            .submodule(engines, &self.mod_path)
            .unwrap_or_else(|| panic!("Could not retrieve submodule for mod_path."))
    }

    /// Initialise the namespace at its root from the given initial namespace.
    /// If the root module contains submodules these are now considered external.
    pub fn init_root(root: &mut Root) -> Self {
        assert!(
            !root.module.is_external,
            "The root module must not be external during compilation"
        );
        let mod_path = vec![];

        // A copy of the root module is used to initialize every new submodule in the program.
        //
        // Every submodule that has been added before calling init_root is now considered
        // external, which we have to enforce at this point.
        fn set_submodules_external(module: &mut Module) {
            for (_, submod) in module.submodules_mut().iter_mut() {
                if !submod.is_external {
                    submod.is_external = true;
                    set_submodules_external(submod);
                }
            }
        }

        set_submodules_external(&mut root.module);
        // The init module itself is not external
        root.module.is_external = false;

        Self {
            init: root.module.clone(),
            root: root.clone(),
            mod_path,
        }
    }

    /// A reference to the path of the module currently being processed.
    pub fn mod_path(&self) -> &ModulePath {
        &self.mod_path
    }

    /// Prepends the module path into the prefixes.
    pub fn prepend_module_path<'a>(
        &'a self,
        prefixes: impl IntoIterator<Item = &'a Ident>,
    ) -> ModulePathBuf {
        self.mod_path.iter().chain(prefixes).cloned().collect()
    }

    /// A reference to the root of the project namespace.
    pub fn root(&self) -> &Root {
        &self.root
    }

    pub fn root_module(&self) -> &Module {
        &self.root.module
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    pub fn module(&self, engines: &Engines) -> &Module {
        self.root
            .module
            .lookup_submodule(&Handler::default(), engines, &self.mod_path)
            .unwrap()
    }

    /// Mutable access to the current [Module], i.e. the module at the inner `mod_path`.
    pub fn module_mut(&mut self, engines: &Engines) -> &mut Module {
        self.root
            .module
            .lookup_submodule_mut(&Handler::default(), engines, &self.mod_path)
            .unwrap()
    }

    /// Returns true if the current module being checked is a direct or indirect submodule of
    /// the module given by the `absolute_module_path`.
    ///
    /// The current module being checked is determined by `mod_path`.
    ///
    /// E.g., the `mod_path` `[fist, second, third]` of the root `foo` is a submodule of the module
    /// `[foo, first]`. Note that the `mod_path` does not contain the root name, while the
    /// `absolute_module_path` always contains it.
    ///
    /// If the current module being checked is the same as the module given by the `absolute_module_path`,
    /// the `true_if_same` is returned.
    pub(crate) fn module_is_submodule_of(
        &self,
        _engines: &Engines,
        absolute_module_path: &ModulePath,
        true_if_same: bool,
    ) -> bool {
        // `mod_path` does not contain the root name, so we have to separately check
        // that the root name is equal to the module package name.
        let root_name = self.root.module.name();

        let (package_name, modules) = absolute_module_path.split_first().expect("Absolute module path must have at least one element, because it always contains the package name.");

        if root_name != package_name {
            return false;
        }

        if self.mod_path.len() < modules.len() {
            return false;
        }

        let is_submodule = modules
            .iter()
            .zip(self.mod_path.iter())
            .all(|(left, right)| left == right);

        if is_submodule {
            if self.mod_path.len() == modules.len() {
                true_if_same
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Returns true if the module given by the `absolute_module_path` is external
    /// to the current package. External modules are imported in the `Forc.toml` file.
    pub(crate) fn module_is_external(&self, absolute_module_path: &ModulePath) -> bool {
        let root_name = self.root.module.name();

        assert!(!absolute_module_path.is_empty(), "Absolute module path must have at least one element, because it always contains the package name.");

        root_name != &absolute_module_path[0]
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(
        &mut self,
        engines: &Engines,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
    ) -> SubmoduleNamespace {
        let init = self.init.clone();
        let is_external = self.module(engines).is_external;
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(mod_name.clone()))
            .collect();
        self.module_mut(engines)
            .submodules
            .entry(mod_name.to_string())
            .or_insert(init.new_submodule_from_init(
                mod_name,
                visibility,
                Some(module_span),
                is_external,
                submod_path.clone(),
            ));
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path.clone());
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }

    /// Pushes a new submodule to the namespace's module hierarchy.
    pub fn push_submodule(
        &mut self,
        engines: &Engines,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
    ) {
        self.module_mut(engines)
            .submodules
            .entry(mod_name.to_string())
            .or_insert(Module::new(mod_name.clone(), visibility, Some(module_span)));
        self.mod_path.push(mod_name.clone());
    }

    /// Pops the current submodule from the namespace's module hierarchy.
    pub fn pop_submodule(&mut self) {
        self.mod_path.pop();
    }
}

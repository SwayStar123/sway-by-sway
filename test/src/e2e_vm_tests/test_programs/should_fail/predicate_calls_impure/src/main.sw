predicate;

// In a script, there can be no impurity since storage is only available in contracts.
#[storage(read, write)]
fn main() {
  foo()
}

#[storage(read,write)]
fn foo() {

}

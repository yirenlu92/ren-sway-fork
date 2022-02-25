script;

fn return_the_same<T>(elem: T) -> T {
  let x: T = elem;
  x
}

fn main() -> u64 {
    let foo = return_the_same(7u64);
    let bar = return_the_same::<u64>(7u64);
   1
}
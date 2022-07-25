use toke;

pub fn main() {
    dbg!(toke::n!(def fn hello world));

    toke::this!().parent().parent()
}

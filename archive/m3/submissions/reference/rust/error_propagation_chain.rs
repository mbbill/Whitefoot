#[derive(Debug, PartialEq, Eq)]
enum ChainErr {
    First,
    Second,
}

fn first(ok: bool) -> Result<i32, ChainErr> {
    if ok { Ok(20) } else { Err(ChainErr::First) }
}

fn second(ok: bool) -> Result<i32, ChainErr> {
    if ok { Ok(22) } else { Err(ChainErr::Second) }
}

fn chain(first_ok: bool, second_ok: bool) -> Result<i32, ChainErr> {
    let a = first(first_ok)?;
    let b = second(second_ok)?;
    Ok(a + b)
}

fn main() {
    assert_eq!(chain(true, true), Ok(42));
    assert_eq!(chain(false, true), Err(ChainErr::First));
    assert_eq!(chain(true, false), Err(ChainErr::Second));
    println!("ok");
}

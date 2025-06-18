pub trait IsZst {
    const ZST: bool;
}
impl<T> IsZst for T {
    const ZST: bool = size_of::<Self>() == 0;
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use super::*;

    struct Transparent(());
    struct Custom;
    struct Empty {}
    struct Unit();

    fn is_zst<T>() -> bool {
        T::ZST
    }

    #[test]
    fn it_works() {
        assert_eq!(false, i32::ZST);
        assert_eq!(false, <[i32; 1]>::ZST);

        assert_eq!(true,  <()>::ZST);
        assert_eq!(true,  <[i32; 0]>::ZST);
        assert_eq!(true,  Infallible::ZST);
        assert_eq!(true,  Transparent::ZST);
        assert_eq!(true,  Custom::ZST);
        assert_eq!(true,  Empty::ZST);
        assert_eq!(true,  Unit::ZST);
    }

    #[test]
    fn test_generic() {
        assert_eq!(false, is_zst::<i32>());
        assert_eq!(false, is_zst::<[i32; 1]>());

        assert_eq!(true,  is_zst::<()>());
        assert_eq!(true,  is_zst::<[i32; 0]>());
        assert_eq!(true,  is_zst::<Infallible>());
        assert_eq!(true,  is_zst::<Transparent>());
        assert_eq!(true,  is_zst::<Custom>());
        assert_eq!(true,  is_zst::<Empty>());
        assert_eq!(true,  is_zst::<Unit>());
    }
}

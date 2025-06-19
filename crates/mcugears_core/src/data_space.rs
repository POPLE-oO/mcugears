// ルートから読み込み
use crate::*;

// RAM操作
pub trait Ram {
    fn new(capacity: RegisterSize) -> Self;
}

#[cfg(test)]
mod test_utilities {
    use super::*;
}

#[cfg(test)]
mod tests {
    // ---  RAMの読み書き  ---
    #[cfg(test)]
    mod test_ram_write_read {
        // ---  write_from,read_fromの実行
        #[test]
        fn test_write_read_from() {}
    }
}

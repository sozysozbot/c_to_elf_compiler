use crate::{x86_64_no_arg::*, Buf};

pub fn rdiから即値を引く(n: i32) -> Buf {
    fn rdiから即値を引く_i8(n: i8) -> [u8; 4] {
        [0x48, 0x83, 0xef, n as u8]
    }

    fn rdiから即値を引く_i32(n: i32) -> [u8; 7] {
        let buf = n.to_le_bytes();
        [0x48, 0x81, 0xef, buf[0], buf[1], buf[2], buf[3]]
    }

    if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        Buf::from(rdiから即値を引く_i8(n as i8))
    } else {
        Buf::from(rdiから即値を引く_i32(n))
    }
}

pub fn raxから即値を引く(n: i32) -> Buf {
    fn raxから即値を引く_i8(n: i8) -> [u8; 4] {
        [0x48, 0x83, 0xe8, n as u8]
    }

    fn raxから即値を引く_i32(n: i32) -> [u8; 6] {
        let buf = n.to_le_bytes();
        [0x48, 0x2d, buf[0], buf[1], buf[2], buf[3]]
    }

    if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        Buf::from(raxから即値を引く_i8(n as i8))
    } else {
        Buf::from(raxから即値を引く_i32(n))
    }
}

pub fn rspから即値を引く(n: i32) -> Buf {
    fn rspから即値を引く_i8(x: i8) -> Buf {
        Buf::from([0x48, 0x83, 0xec, x as u8])
    }
    fn rspから即値を引く_i32(x: i32) -> Buf {
        let buf = x.to_le_bytes();
        Buf::from([0x48, 0x81, 0xec, buf[0], buf[1], buf[2], buf[3]])
    }
    if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        rspから即値を引く_i8(n as i8)
    } else {
        rspから即値を引く_i32(n)
    }
}

pub fn rspに即値を足す(n: i32) -> Buf {
    fn rspに即値を足す_i8(x: i8) -> Buf {
        Buf::from([0x48, 0x83, 0xc4, x as u8])
    }
    fn rspに即値を足す_i32(x: i32) -> Buf {
        let buf = x.to_le_bytes();
        Buf::from([0x48, 0x81, 0xc4, buf[0], buf[1], buf[2], buf[3]])
    }
    if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        rspに即値を足す_i8(n as i8)
    } else {
        rspに即値を足す_i32(n)
    }
}

pub fn プロローグ(x: i32) -> Buf {
    Buf::from(rbpをプッシュ())
        .join(rspをrbpにコピー())
        .join(rspから即値を引く(x))
}

pub fn jmp(n: i32) -> Buf {
    fn _jmp_i8(n: i8) -> Buf {
        Buf::from([0xeb, n.to_le_bytes()[0]])
    }

    fn jmp_i32(n: i32) -> Buf {
        let buf = n.to_le_bytes();
        Buf::from([0xe9, buf[0], buf[1], buf[2], buf[3]])
    }

    /*if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        _jmp_i8(n as i8)
    } else*/ {
        jmp_i32(n)
    }
}

pub fn je(n: i32) -> Buf {
    fn je_i8(n: i8) -> Buf {
        Buf::from([0x74, n.to_le_bytes()[0]])
    }

    fn je_i32(n: i32) -> Buf {
        let buf = n.to_le_bytes();
        Buf::from([0x0f, 0x84, buf[0], buf[1], buf[2], buf[3]])
    }

    if n >= i8::MIN as i32 && n <= i8::MAX as i32 {
        je_i8(n as i8)
    } else {
        je_i32(n)
    }
}

pub fn ediに代入(n: u32) -> [u8; 5] {
    let buf = n.to_le_bytes();
    [0xbf, buf[0], buf[1], buf[2], buf[3]]
}

pub fn eaxに即値をセット(n: u32) -> [u8; 5] {
    let buf = n.to_le_bytes();
    [0xb8, buf[0], buf[1], buf[2], buf[3]]
}

pub fn edxに即値をセット(n: u32) -> [u8; 5] {
    let buf = n.to_le_bytes();
    [0xba, buf[0], buf[1], buf[2], buf[3]]
}

pub fn rbpにoffsetを足した位置にdilを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にdilを代入_i8(offset: i8) -> Buf {
        Buf::from([0x40, 0x88, 0x7d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にdilを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x40, 0x88, 0xbd, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にdilを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にdilを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にsilを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にsilを代入_i8(offset: i8) -> Buf {
        Buf::from([0x40, 0x88, 0x75, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にsilを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x40, 0x88, 0xb5, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にsilを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にsilを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にdlを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にdlを代入_i8(offset: i8) -> Buf {
        Buf::from([0x88, 0x55, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にdlを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x88, 0x95, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にdlを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にdlを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にclを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にclを代入_i8(offset: i8) -> Buf {
        Buf::from([0x88, 0x4d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にclを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x88, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にclを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にclを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr8bを代入(offset: i32) -> Buf {
    pub fn rbpにoffsetを足した位置にr8bを代入_i8(offset: i8) -> Buf {
        Buf::from([0x44, 0x88, 0x45, offset.to_le_bytes()[0]])
    }
    pub fn rbpにoffsetを足した位置にr8bを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x44, 0x88, 0x85, buf[0], buf[1], buf[2], buf[3]])
    }
    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr8bを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr8bを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr9bを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にr9bを代入_i8(offset: i8) -> Buf {
        Buf::from([0x44, 0x88, 0x4d, offset.to_le_bytes()[0]])
    }
    fn rbpにoffsetを足した位置にr9bを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x44, 0x88, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }
    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr9bを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr9bを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にediを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にediを代入_i8(offset: i8) -> Buf {
        Buf::from([0x89, 0x7d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にediを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x89, 0xbd, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にediを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にediを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にesiを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にesiを代入_i8(offset: i8) -> Buf {
        Buf::from([0x89, 0x75, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にesiを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x89, 0xb5, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にesiを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にesiを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にedxを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にedxを代入_i8(offset: i8) -> Buf {
        Buf::from([0x89, 0x55, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にedxを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x89, 0x95, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にedxを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にedxを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にecxを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にecxを代入_i8(offset: i8) -> Buf {
        Buf::from([0x89, 0x4d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にecxを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x89, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にecxを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にecxを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr8dを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にr8dを代入_i8(offset: i8) -> Buf {
        Buf::from([0x44, 0x89, 0x45, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にr8dを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x44, 0x89, 0x85, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr8dを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr8dを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr9dを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にr9dを代入_i8(offset: i8) -> Buf {
        Buf::from([0x44, 0x89, 0x4d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にr9dを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x44, 0x89, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr9dを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr9dを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にrdiを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にrdiを代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x7d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にrdiを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0xbd, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にrdiを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にrdiを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にrsiを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にrsiを代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x75, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にrsiを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0xb5, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にrsiを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にrsiを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にrdxを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にrdxを代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x55, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にrdxを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0x95, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にrdxを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にrdxを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にrcxを代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にrcxを代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x4d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にrcxを代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にrcxを代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にrcxを代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr8を代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にr8を代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x45, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にr8を代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0x85, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr8を代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr8を代入_i32(offset)
    }
}

pub fn rbpにoffsetを足した位置にr9を代入(offset: i32) -> Buf {
    fn rbpにoffsetを足した位置にr9を代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x89, 0x4d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足した位置にr9を代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x89, 0x8d, buf[0], buf[1], buf[2], buf[3]])
    }

    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足した位置にr9を代入_i8(offset as i8)
    } else {
        rbpにoffsetを足した位置にr9を代入_i32(offset)
    }
}

pub fn rbpにoffsetを足したアドレスをrdiに代入(offset: i32) -> Buf {
    fn rbpにoffsetを足したアドレスをrdiに代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x8d, 0x7d, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足したアドレスをrdiに代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x8d, 0xbd, buf[0], buf[1], buf[2], buf[3]])
    }
    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足したアドレスをrdiに代入_i8(offset as i8)
    } else {
        rbpにoffsetを足したアドレスをrdiに代入_i32(offset)
    }
}

pub fn rbpにoffsetを足したアドレスをrsiに代入(offset: i32) -> Buf {
    fn rbpにoffsetを足したアドレスをrsiに代入_i8(offset: i8) -> Buf {
        Buf::from([0x48, 0x8d, 0x75, offset.to_le_bytes()[0]])
    }

    fn rbpにoffsetを足したアドレスをrsiに代入_i32(offset: i32) -> Buf {
        let buf = offset.to_le_bytes();
        Buf::from([0x48, 0x8d, 0xb5, buf[0], buf[1], buf[2], buf[3]])
    }
    if offset >= i8::MIN as i32 && offset <= i8::MAX as i32 {
        rbpにoffsetを足したアドレスをrsiに代入_i8(offset as i8)
    } else {
        rbpにoffsetを足したアドレスをrsiに代入_i32(offset)
    }
}

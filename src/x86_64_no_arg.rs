use crate::Buf;

pub fn rdiをプッシュ() -> [u8; 1] {
    [0x57]
}

pub fn rbpをプッシュ() -> [u8; 1] {
    [0x55]
}

pub fn rspをrbpにコピー() -> [u8; 3] {
    [0x48, 0x89, 0xe5]
}

pub fn rsiをraxにコピー() -> [u8; 3] {
    [0x48, 0x89, 0xf0]
}


// leave = mov rsp, rbp + pop rbp
pub fn leave() -> Buf {
    Buf::from([0xc9])
}

pub fn エピローグ() -> Buf {
    leave().join(ret())
}

pub fn rdiへとポップ() -> [u8; 1] {
    [0x5f]
}

pub fn rsiへとポップ() -> [u8; 1] {
    [0x5e]
}

pub fn raxへとポップ() -> [u8; 1] {
    [0x58]
}

pub fn rdxへとポップ() -> [u8; 1] {
    [0x5a]
}

pub fn rcxへとポップ() -> [u8; 1] {
    [0x59]
}

pub fn r8へとポップ() -> [u8; 2] {
    [0x41, 0x58]
}

pub fn r9へとポップ() -> [u8; 2] {
    [0x41, 0x59]
}

pub fn rdiにraxを足し合わせる() -> [u8; 3] {
    [0x48, 0x01, 0xc7]
}

pub fn rdiからraxを減じる() -> [u8; 3] {
    [0x48, 0x29, 0xc7]
}

pub fn rdiをrax倍にする() -> [u8; 4] {
    [0x48, 0x0f, 0xaf, 0xf8]
}

pub fn eaxの符号ビットをedxへ拡張() -> [u8; 1] {
    [0x99]
}

pub fn edx_eaxをediで割る_商はeaxに_余りはedxに() -> [u8; 2] {
    [0xf7, 0xff]
}

pub fn raxをプッシュ() -> [u8; 1] {
    [0x50]
}

pub fn rdxをプッシュ() -> [u8; 1] {
    [0x52]
}

pub fn eaxとediを比較してフラグをセット() -> [u8; 2] {
    [0x39, 0xf8]
}

pub fn フラグを読んで等しいかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x94, 0xc0]
}

pub fn フラグを読んで異なっているかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x95, 0xc0]
}

pub fn フラグを読んで未満であるかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x9c, 0xc0]
}

pub fn フラグを読んで以下であるかどうかをalにセット() -> [u8; 3] {
    [0x0f, 0x9e, 0xc0]
}

pub fn alをゼロ拡張してediにセット() -> [u8; 3] {
    [0x0f, 0xb6, 0xf8]
}

pub fn rdiを間接参照() -> [u8; 3] {
    [0x48, 0x8b, 0x3f]
}

pub fn rdiをmovzxで間接参照() -> [u8; 4] {
    [0x48, 0x0f, 0xb6, 0x3f]
}

pub fn raxが指す位置にrdiを代入() -> [u8; 3] {
    [0x48, 0x89, 0x38]
}

pub fn raxが指す位置にediを代入() -> [u8; 2] {
    [0x89, 0x38]
}

pub fn raxが指す位置にdilを代入() -> [u8; 3] {
    [0x40, 0x88, 0x38]
}

pub fn dilが0かを確認() -> [u8; 4] {
    [0x40, 0x80, 0xff, 0x00]
}

pub fn ediが0かを確認() -> [u8; 3] {
    [0x83, 0xff, 0x00]
}

pub fn rdiが0かを確認() -> [u8; 4] {
    [0x48, 0x83, 0xff, 0x00]
}

pub fn call_rax() -> [u8; 2] {
    [0xff, 0xd0]
}

pub fn syscall() -> [u8; 2] {
    [0x0f, 0x05]
}

pub fn ret() -> [u8; 1] {
    [0xc3]
}

pub fn eaxをediにコピー() -> [u8; 2] {
    [0x89, 0xc7]
}

pub fn raxをrdiにコピー() -> [u8; 3] {
    [0x48, 0x89, 0xc7]
}

pub fn alをediに符号拡張してmov() -> [u8; 3] {
    [0x0f, 0xbe, 0xf8]
}

pub fn ediをeaxにコピー() -> [u8; 2] {
    [0x89, 0xf8]
}

pub fn rdiをraxにコピー() -> [u8; 3] {
    [0x48, 0x89, 0xf8]
}

pub fn dilをeaxに符号拡張してmov() -> [u8; 4] {
    [0x40, 0x0f, 0xbe, 0xc7]
}

pub fn leave_ret() -> [u8; 2] {
    [0xc9, 0xc3]
}
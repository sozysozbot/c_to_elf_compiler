/*
RopeのようなO(1)で連結できるバッファ
*/

#[derive(Debug, Clone)]
pub enum Buf {
    Leaf {
        buf: Vec<u8>,
    },
    Branch {
        left: Box<Buf>,
        right: Box<Buf>,
        len: usize,
    },
}

impl Default for Buf {
    fn default() -> Self {
        Self::new()
    }
}

impl Buf {
    pub fn new() -> Self {
        Self::from(Vec::new())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.write_to_vec(&mut vec);
        vec
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            Self::Leaf { buf } => buf.is_empty(),
            Self::Branch { len, .. } => *len == 0,
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            Self::Leaf { buf } => buf.len(),
            Self::Branch { len, .. } => *len,
        }
    }

    #[must_use]
    pub fn join(self, other: impl Into<Buf>) -> Self {
        let other = other.into();
        let len = self.len() + other.len();

        Self::Branch {
            left: Box::new(self),
            right: Box::new(other),
            len,
        }
    }

    pub fn append(&mut self, other: impl Into<Buf>) {
        let buf = std::mem::take(self);
        let buf = buf.join(other);
        *self = buf;
    }

    fn write_to_vec(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Leaf { buf: leaf_buf } => {
                buf.extend_from_slice(leaf_buf);
            }
            Self::Branch {
                left,
                right,
                len: _,
            } => {
                left.write_to_vec(buf);
                right.write_to_vec(buf);
            }
        }
    }
}

impl From<Vec<u8>> for Buf {
    fn from(buf: Vec<u8>) -> Self {
        Self::Leaf { buf }
    }
}

impl From<&[u8]> for Buf {
    fn from(buf: &[u8]) -> Self {
        Self::Leaf { buf: buf.to_vec() }
    }
}

impl<const N: usize> From<[u8; N]> for Buf {
    fn from(buf: [u8; N]) -> Self {
        Self::Leaf { buf: buf.to_vec() }
    }
}

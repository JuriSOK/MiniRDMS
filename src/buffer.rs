use bytebuffer::ByteBuffer;
use std::cell::RefCell;
use std::cell::RefMut;
use std::io::Write;
use std::io::{Error, ErrorKind, Result};
use std::rc::Rc;
use std::str;
pub struct Buffer {
    buffer: Rc<RefCell<ByteBuffer>>,
}

pub fn check_space(buf: &ByteBuffer, lastpos: usize) -> Result<()> {
    if lastpos > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "buffer overflows due to this operation",
        ));
    }
    Ok(())
}

impl Buffer {
    pub fn new(buff: &Rc<RefCell<ByteBuffer>>) -> Buffer {
        Buffer {
            buffer: Rc::clone(buff),
        }
    }

    pub fn write_int(&mut self, pos: usize, val: i32) -> Result<()> {
        let mut buf = self.buffer.borrow_mut();
        check_space(&buf, pos + 4)?;

        buf.set_wpos(pos);
        buf.write_i32(val);
        buf.reset_cursors();

        Ok(())
    }

    pub fn read_int(&self, pos: usize) -> Result<i32> {
        let mut buf = self.buffer.borrow_mut();

        buf.set_rpos(pos);
        buf.read_i32()
    }

    pub fn write_float(&mut self, pos: usize, val: f32) -> Result<()> {
        let mut buf = self.buffer.borrow_mut();
        check_space(&buf, pos + 4)?;

        buf.set_wpos(pos);
        buf.write_f32(val);
        buf.reset_cursors();

        Ok(())
    }

    pub fn read_float(&self, pos: usize) -> Result<f32> {
        let mut buf = self.buffer.borrow_mut();
        buf.set_rpos(pos);
        buf.read_f32()
    }

    pub fn write_string(&mut self, pos: usize, val: &str, size: usize) -> Result<()> {
        let mut buf = self.buffer.borrow_mut();
        check_space(&buf, val.len())?;
        buf.set_wpos(pos);

        let bytes = val.as_bytes();
        buf.write_all(&bytes[..size])?;

        Ok(())
    }

    pub fn read_string(&self, pos: usize, size: usize) -> Result<String> {
        let mut buf = self.buffer.borrow_mut();
        buf.set_rpos(pos);

        let bytes = buf.read_bytes(size)?;
        let string_value = String::from_utf8(bytes);

        Ok(string_value.unwrap())
    }

    pub fn get_mut_buffer(&self) -> RefMut<'_, ByteBuffer> {
        self.buffer.borrow_mut()
    }
}

#[cfg(test)]
mod tests {
    use bytebuffer::ByteBuffer;
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::buffer::Buffer;
    #[test]
    fn test_write_read_int() {
        let mut buffer = ByteBuffer::new();
        buffer.resize(32);
        let refcbuffer = RefCell::new(buffer);
        let mut buffer2 = Buffer::new(&Rc::new(refcbuffer));

        let _ = buffer2.write_int(4, 10);
        println!("{:?}", buffer2.buffer.borrow());
        assert_eq!(buffer2.read_int(4).unwrap(), 10);
    }

    #[test]
    fn test_write_read_float() {
        let mut buffer = ByteBuffer::new();
        buffer.resize(32);
        let refcbuffer = RefCell::new(buffer);
        let mut buffer2 = Buffer::new(&Rc::new(refcbuffer));

        let _ = buffer2.write_float(2, 10.3);
        println!("{:?}", buffer2.buffer.borrow());
        assert_eq!(buffer2.read_float(2).unwrap(), 10.3);
    }

    #[test]
    fn test_write_read_string() {
        let mut buffer = ByteBuffer::new();
        buffer.resize(32);

        let refcbuffer = RefCell::new(buffer);
        let mut buffer2 = Buffer::new(&Rc::new(refcbuffer));

        let _ = buffer2.write_string(0, "Salut moi cest Arnaud", "Salut moi cest Arnaud".len());
        assert_eq!(
            buffer2
                .read_string(0, "Salut moi cest Arnaud".len())
                .unwrap(),
            "Salut moi cest Arnaud"
        );
    }
}

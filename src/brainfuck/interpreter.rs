const MEMSIZE: usize = 256;

use super::Io;

fn exec<T: Io>(ins: &[u8], mem: &mut [u8], cur:usize, pos:usize, io: &mut T) -> usize
{
    let mut cur = cur;
    let mut pos = pos;
    loop {
        match ins.get(pos) {
            Some(&b'<') => { cur = (cur + mem.len() - 1)%MEMSIZE; }
            Some(&b'>') => { cur = (cur + mem.len() + 1)%MEMSIZE; }
            Some(&b'-') => { mem[cur] -= 1; }
            Some(&b'+') => { mem[cur] += 1; }
            Some(&b'.') => { io.write_byte(mem[cur]); }
            Some(&b',') => { mem[cur] = io.read_byte(); }
            Some(&b'[') => {
                while mem[cur] != 0 {
                    cur = exec(ins, mem, cur, pos+1, io);
                }
                pos = skip_loop(ins, pos+1);
            }
            Some(&b']') | None => { break; }
            _ => {/*comment*/}
        }
        pos += 1;
    }
    cur
}

fn skip_loop(ins: &[u8], pos: usize) -> usize {
    // TODO: rewrite without recursion
    let mut pos = pos;
    loop {
        match ins.get(pos) {
            Some(&b'[') => {
                pos = skip_loop(ins, pos+1);
            },
            Some(&b']') | None => {
                return pos;
            },
            _ => {},
        }
        pos += 1;
    }
}

pub fn interpret<T: Io>(s: &[u8], io: &mut T) {
    let mut memory = [0u8; MEMSIZE];
    exec(s, &mut memory, 0, 0, io);
}

#[cfg(test)]
mod test {
    use super::super::Io;

    struct TestIO<'a> {
        expected_in: &'a [u8],
        expected_out: &'a [u8],
        cur_in: usize,
        cur_out: usize,
    }

    impl<'a> Io for TestIO<'a> {
        fn write_byte(&mut self, byte: u8) {
            assert!(self.expected_out.len() != self.cur_out);
            assert_eq!(self.expected_out[self.cur_out], byte);
            self.cur_out += 1;
        }

        fn read_byte(&mut self) -> u8 {
            assert!(self.expected_in.len() != self.cur_in);
            self.cur_in += 1;
            self.expected_in[self.cur_in - 1]
        }
    }

    fn test(cmd: &[u8], input: &[u8], output: &[u8]) {
        let mut memory = [0u8; 256];

        let mut io = TestIO {
            expected_in: input,
            expected_out: output,
            cur_in: 0,
            cur_out: 0,
        };
        super::exec(cmd, &mut memory, 0, 0, &mut io);
        assert_eq!(input.len(), io.cur_in);
        assert_eq!(output.len(), io.cur_out);
    }

    #[test]
    fn test_id() {
        test(b",.", b"j", b"j");
    }

    #[test]
    fn test_inc() {
        test(b".+.+.+.+.", b"", &[0,1,2,3,4]);
    }

    #[test]
    fn test_dec() {
        test(b"+++.-.-.-.", b"", &[3,2,1,0]);
    }

    #[test]
    fn test_dont_loop() {
        test(b"[]", b"", b"");
    }

    #[test]
    fn test_dont_loop_exit() {
        test(b"[].", b"", &[0]);
    }

    #[test]
    fn test_loop_one() {
        test(b"+.[-].", b"", &[1,0]);
    }

    #[test]
    fn test_loop_fixed() {
        test(b"+++.[-].", b"", &[3,0]);
    }

    #[test]
    fn test_move() {
        test(b",[>+<-]>.", b"k", b"k");
    }

    #[test]
    fn test_a() {
        test(b"++++++[>++++++++++<-]>+++++.", b"", b"A");
    }

    #[test]
    fn test_hello_world() {
        test(b"++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.", b"", b"Hello World!");
    }

    #[test]
    fn test_hello_world2() {
        test(b"++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.", b"", b"Hello World!");
    }
}

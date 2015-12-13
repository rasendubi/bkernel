use collections::Vec;

const MEMSIZE: usize = 0xfe;

use super::{Io};

fn exec(ins: &Vec<char>, mem: &mut [u8], cur:usize, pos:usize, io: &Io) -> usize
{
    let mut cur = cur;
    let mut pos = pos;
    loop {
        match ins.get(pos) {
            Some(&'<') => {cur=(cur-1)%MEMSIZE;}
            Some(&'>') => {cur=(cur+1)%MEMSIZE;}
            Some(&'-') => {mem[cur]-=1;}
            Some(&'+') => {mem[cur]+=1;}
            Some(&'.') => {io.write_byte(mem[cur]);}
            Some(&',') => {mem[cur]=io.read_byte();}
            Some(&'[') => {
                while mem[cur] != 0 {
                    cur = exec(ins, mem, cur, pos+1, io);
                }
                while ins.get(pos) != Some(&']') {pos+=1}
            }
            Some(&']') | None => {break;}
            _ => {/*comment*/}
        }
        pos += 1;
    }
    cur
}

pub fn interpret(s: &str, io: &Io) {
    let s = s.chars().collect();
    exec(&s, &mut [0u8; MEMSIZE], 0, 0, io);
}

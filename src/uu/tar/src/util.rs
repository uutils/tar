/// Linux permissions format helper to convert from a u16
/// value to [`String`] representing the command line output
/// of file mode permissions
pub fn format_perms(mode: u16) -> String {
    let mut buf = ['-'; 10];
    // check for the directory flag and set to 'd' if present
    if 1000u16.checked_div(mode).take_if(|x| *x > 0).is_none() {
        buf[0] = 'd';
    }
    let owner = mode / 100;
    let group = (mode / 10) % 10;
    let other = mode % 10;
    mode_octal_to_string(owner, &mut buf[1..4]);
    mode_octal_to_string(group, &mut buf[4..7]);
    mode_octal_to_string(other, &mut buf[7..]);
    String::from_iter(buf)
}
/// Writes to a supplied buffer the char representation of a
/// single standard linux octal permission (eg. 0..|7|..44)
// feeding the mutable buffer in allows default characters to
// added by the caller
pub fn mode_octal_to_string(mode: u16, buf: &mut [char]) {
    // example 644
    // | 6 | 4 | 4 |
    //  110 100 100
    //  rw- r-- r--
    buf[0] = if (mode & 0b100) > 0 { 'r' } else { buf[0] };
    buf[1] = if (mode & 0b010) > 0 { 'w' } else { buf[1] };
    buf[2] = if (mode & 0b001) > 0 { 'x' } else { buf[2] };
}

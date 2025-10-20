pub fn format_perms(mode: u16) -> String {
    let mut buf = ['-'; 10];
    if let None = 1000u16.checked_div(mode).take_if(|x| *x > 0) {
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
/// Formats stand linux octal permissions (eg. 0744)
pub fn mode_octal_to_string(mode: u16, buf: &mut [char]) {
    // example 644
    // | 6 | 4 | 4 |
    //  110 100 100
    //  rw- r-- r--
    buf[0] = if (mode & 0b100) > 0 {'r'}else{'-'};
    buf[1] = if (mode & 0b010) > 0 {'w'}else{'-'};
    buf[2] = if (mode & 0b001) > 0 {'x'}else{'-'};
}

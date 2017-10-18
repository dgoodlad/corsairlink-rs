use nom::{IResult,be_u8,le_u16};

#[derive(Debug,PartialEq,Eq)]
struct Command {
    command_id: u8,
    response: Response,
}

#[derive(Debug,PartialEq,Eq)]
enum Response {
    Ack(u8),
    ReadOneByte(u8),
    ReadTwoBytes(u16),
    ReadManyBytes(Vec<u8>),
}

named!(commands<Vec<Command>>,
    many1!(command)
);

named!(command<Command>,
    do_parse!(
        id: be_u8     >>
        response: ack >>
        (Command {
            command_id: id,
            response: response
        })
    )
);

named!(ack<Response>, alt!(
    write |
    read_one_byte |
    read_two_bytes |
    read_many_bytes
));

named!(write<Response>, 
    do_parse!(
        opcode: alt!(tag!(&[0x6u8][..]) | tag!(&[0x8u8][..]) | tag!(&[0xAu8][..])) >>
        (Response::Ack(opcode[0]))
    )
);

named!(read_one_byte<Response>,
    do_parse!(
        tag!( &[0x7u8][..] ) >>
        b: be_u8 >>
        (Response::ReadOneByte(b))
    )
);

named!(read_two_bytes<Response>,
    do_parse!(
        tag!( &[0x9u8][..] ) >>
        n: le_u16 >>
        (Response::ReadTwoBytes(n))
    )
);

named!(read_many_bytes<Response>,
    do_parse!(
        tag!(&[0xBu8][..]) >>
        data: length_bytes!(be_u8) >>
        (Response::ReadManyBytes(data.to_vec()))
    )
);

#[cfg(test)]
mod tests {
    use parser::*;

    #[test]
    fn test_write1() {
        let empty = &b""[..];
        let input = [0x6u8];
        let expected = IResult::Done(empty, Response::Ack(0x6));
        let res = write(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_write2() {
        let empty = &b""[..];
        let input = [0x8u8];
        let expected = IResult::Done(empty, Response::Ack(0x8));
        let res = write(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_write3() {
        let empty = &b""[..];
        let input = [0xAu8];
        let expected = IResult::Done(empty, Response::Ack(0xa));
        let res = write(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_read_one_byte1() {
        let empty = &b""[..];
        let input = [0x7u8, 0x42];
        let expected = IResult::Done(empty, Response::ReadOneByte(0x42));
        let res = read_one_byte(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_read_two_bytes1() {
        let empty = &b""[..];
        let input = [0x9u8, 0x42, 0x10];
        let expected = IResult::Done(empty, Response::ReadTwoBytes(0x1042));
        let res = read_two_bytes(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_read_many_bytes() {
        let leftover = &[0u8;3][..];
        let input = [0x0Bu8, 0x04, 0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00];
        let expected = IResult::Done(leftover, Response::ReadManyBytes(vec![0x01u8, 0x02, 0x03, 0x04]));
        let res = read_many_bytes(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_ack() {
        let empty = &b""[..];
        let input = [0x9u8, 0x42, 0x10];
        let expected = IResult::Done(empty, Response::ReadTwoBytes(0x1042));
        let res = ack(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_command() {
        let empty = &b""[..];
        let input = [0x10u8, 0x9, 0x42, 0x10];
        let expected = IResult::Done(empty, Command { command_id: 0x10, response: Response::ReadTwoBytes(0x1042) });
        let res = command(&input);
        assert_eq!(res, expected)
    }

    #[test]
    fn test_commands() {
        let empty = &b""[..];
        let input = [0x10u8, 0x9, 0x42, 0x10,
                     0x11,   0x7, 0x1];
        let expected = IResult::Done(empty, vec![
            Command { command_id: 0x10, response: Response::ReadTwoBytes(0x1042) },
            Command { command_id: 0x11, response: Response::ReadOneByte(0x1)}
        ]);
        let res = commands(&input);
        assert_eq!(res, expected)
    }
}
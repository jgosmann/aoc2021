use hex;
use std::{
    error::Error,
    io::{self, BufRead},
};

#[derive(Debug, Clone)]
struct BitReader<'b> {
    buf: &'b [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'b> BitReader<'b> {
    fn new(buf: &'b [u8]) -> Self {
        Self {
            buf,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    fn read_bits(&mut self, mut n: usize) -> usize {
        let mut bits: usize = 0;

        while n > 0 {
            let byte = self.buf[self.byte_pos];

            let to_read = usize::min(n, (8u8 - self.bit_pos) as usize) as u8;
            bits = (bits << to_read)
                | ((byte & (u8::MAX >> self.bit_pos)) >> (8 - self.bit_pos - to_read)) as usize;
            n -= to_read as usize;
            self.bit_pos += to_read;
            if self.bit_pos >= 8 {
                self.byte_pos += 1;
                self.bit_pos = 0;
            }
        }

        bits
    }

    fn bits_consumed(&self) -> usize {
        self.byte_pos * 8 + self.bit_pos as usize
    }

    fn is_eof(&self) -> bool {
        self.byte_pos == self.buf.len() - 1
            && (self.buf[self.byte_pos] & (u8::MAX >> self.bit_pos) == 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Version(u8);

impl Version {
    fn read(reader: &mut BitReader) -> Self {
        Self(reader.read_bits(3) as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Literal,
    Sum,
    Product,
    Min,
    Max,
    GreaterThan,
    LessThan,
    Eq,
}

impl Type {
    fn read(reader: &mut BitReader) -> Self {
        match reader.read_bits(3) as u8 {
            0 => Type::Sum,
            1 => Type::Product,
            2 => Type::Min,
            3 => Type::Max,
            4 => Type::Literal,
            5 => Type::GreaterThan,
            6 => Type::LessThan,
            7 => Type::Eq,
            _ => panic!("3 bits should never be larger than 7."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Literal(usize);

impl Literal {
    fn read(reader: &mut BitReader) -> Self {
        let mut value: usize = 0;
        let mut segment = 0b10000;
        while segment & 0b10000 > 0 {
            segment = reader.read_bits(5);
            value = (value << 4) ^ (segment & 0b1111);
        }
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Packet {
    version: Version,
    packet_type: Type,
    sub_packets: Vec<Packet>,
    data: Option<Literal>,
}

impl Packet {
    pub fn read(reader: &mut BitReader) -> Self {
        let version = Version::read(reader);
        let packet_type = Type::read(reader);
        let sub_packets = match packet_type {
            Type::Literal => vec![],
            _ => {
                let length_type_id = reader.read_bits(1);
                match length_type_id {
                    0 => Self::read_fixed_bit_length_sub_packets(reader),
                    1 => Self::read_fixed_packet_length_sub_packets(reader),
                    _ => panic!("This should never happen."),
                }
            }
        };
        let data = match packet_type {
            Type::Literal => Some(Literal::read(reader)),
            _ => None,
        };
        Self {
            version,
            packet_type,
            sub_packets,
            data,
        }
    }

    fn read_fixed_bit_length_sub_packets(reader: &mut BitReader) -> Vec<Packet> {
        let bit_length = reader.read_bits(15);

        let mut bits_to_read = bit_length;
        let mut buf: Vec<u8> = vec![];
        while bits_to_read > 0 {
            let n = usize::min(8, bits_to_read);
            buf.push((reader.read_bits(n) as u8) << (8 - n));
            bits_to_read -= n;
        }

        let mut sub_reader = BitReader::new(&buf);
        let mut packets = vec![];
        while !sub_reader.is_eof() && sub_reader.bits_consumed() < bit_length {
            packets.push(Packet::read(&mut sub_reader));
        }
        packets
    }

    fn read_fixed_packet_length_sub_packets(reader: &mut BitReader) -> Vec<Packet> {
        let packet_length = reader.read_bits(11);
        (0..packet_length)
            .into_iter()
            .map(|_| Packet::read(reader))
            .collect()
    }

    fn version_sum(&self) -> u64 {
        self.version.0 as u64 + self.sub_packets.iter().map(Self::version_sum).sum::<u64>()
    }

    fn eval(&self) -> u64 {
        match self.packet_type {
            Type::Literal => self.data.unwrap().0 as u64,
            Type::Sum => self.sub_packets.iter().map(Self::eval).sum(),
            Type::Product => self.sub_packets.iter().map(Self::eval).product(),
            Type::Min => self
                .sub_packets
                .iter()
                .map(Self::eval)
                .min()
                .unwrap_or_default(),
            Type::Max => self
                .sub_packets
                .iter()
                .map(Self::eval)
                .max()
                .unwrap_or_default(),
            Type::GreaterThan => {
                if self.sub_packets[0].eval() > self.sub_packets[1].eval() {
                    1
                } else {
                    0
                }
            }
            Type::LessThan => {
                if self.sub_packets[0].eval() < self.sub_packets[1].eval() {
                    1
                } else {
                    0
                }
            }
            Type::Eq => {
                if self.sub_packets[0].eval() == self.sub_packets[1].eval() {
                    1
                } else {
                    0
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = stdin
        .lock()
        .lines()
        .next()
        .unwrap_or(Ok(String::default()))?;
    let buf = hex::decode(&input)?;
    let mut bit_reader = BitReader::new(&buf);
    let packet = Packet::read(&mut bit_reader);
    println!("Sum of versions: {}", packet.version_sum());
    println!("Evaluation: {}", packet.eval());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("8A004A801A8002F478", 16)]
    #[case("620080001611562C8802118E34", 12)]
    #[case("C0015000016115A2E0802F182340", 23)]
    #[case("A0016C880162017C3686B18A3D4780", 31)]
    fn test_version_sum(#[case] hex_input: &str, #[case] expected: u64) {
        let input = hex::decode(hex_input).unwrap();
        let mut bit_reader = BitReader::new(&input);
        assert_eq!(Packet::read(&mut bit_reader).version_sum(), expected);
    }

    #[rstest]
    #[case("C200B40A82", 3)]
    #[case("04005AC33890", 54)]
    #[case("880086C3E88112", 7)]
    #[case("CE00C43D881120", 9)]
    #[case("D8005AC2A8F0", 1)]
    #[case("F600BC2D8F", 0)]
    #[case("9C005AC2F8F0", 0)]
    #[case("9C0141080250320F1802104A08", 1)]
    fn test_evaluation(#[case] hex_input: &str, #[case] expected: u64) {
        let input = hex::decode(hex_input).unwrap();
        let mut bit_reader = BitReader::new(&input);
        assert_eq!(Packet::read(&mut bit_reader).eval(), expected);
    }
}

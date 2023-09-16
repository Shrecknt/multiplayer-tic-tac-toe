use std::env;
use std::fs;
use std::path::Path;

use serde_json::{from_slice, Value};

const DEBUG_PACKETS: bool = false;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("packets.rs");

    let source: Value = from_slice(&fs::read("packets.json").unwrap()).unwrap();
    let packets_obj = source.as_object().unwrap();

    let mut codegen = String::from("use async_trait::async_trait;\n\npub type DynamicRead<'a> = (dyn tokio::io::AsyncRead + Unpin + Send + Sync + 'a);\npub type DynamicWrite<'a> = (dyn tokio::io::AsyncWrite + Unpin + Send + Sync + 'a);\n\nasync fn read_varint(reader: &mut DynamicRead<'_>) -> Option<i32> {\n    use tokio::io::AsyncReadExt;\n    let mut buffer = [0];\n    let mut ans = 0;\n    for i in 0..5 {\n        reader.read_exact(&mut buffer).await.ok()?;\n        ans |= ((buffer[0] & 0b0111_1111) as i32) << (7 * i);\n        if buffer[0] & 0b1000_0000 == 0 {\n            return Some(ans);\n        }\n    }\n    Some(ans)\n}\n\npub fn read_packet_string<T: std::io::Read>(buf: &mut T) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {\n    use integer_encoding::VarIntReader;\n    let string_length: u32 = buf.read_varint()?;\n    let mut res = vec![0u8; string_length as usize];\n    buf.read_exact(&mut res)?;\n    Ok(std::str::from_utf8(&res)?.to_string())\n}\n\npub fn write_packet_string<T: std::io::Write>(\n    buf: &mut T,\n    src: &str,\n) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n    use integer_encoding::VarIntWriter;\n    let string_bytes = src.as_bytes();\n    let string_length = string_bytes.len();\n    buf.write_varint(string_length)?;\n    buf.write_all(string_bytes)?;\n    Ok(())\n}\n\nfn prepend_length(buf: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {\n    use integer_encoding::VarIntWriter;\n    use std::io::Write;\n    let mut res: Vec<u8> = vec![];\n    res.write_varint(buf.len())?;\n    res.write_all(buf)?;\n    Ok(res)\n}\n\n#[async_trait]\npub trait Packet {\n    fn packet_id(&self) -> u32;\n    fn deserialize_slice<T: AsRef<[u8]>>(buf: &mut T) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>\n    where\n        Self: Sized;\n    async fn deserialize(stream: &mut DynamicRead<'_>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>\n    where\n        Self: Sized;\n    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;\n}\n\n");

    for (stage, value) in packets_obj {
        let mut packet_id: u32 = 0;
        for (direction, packets) in value.as_object().unwrap() {
            let enum_name = format!("{direction}{stage}Packet");
            let mut enum_str = format!("#[derive(Clone, Debug)]\npub enum {enum_name} {{\n");
            let mut enum_impl = format!("#[async_trait]\nimpl Packet for {enum_name} {{\n");
            let mut impl_packet_id =
                "    fn packet_id(&self) -> u32 {\n        match self {\n".to_string();
            let mut impl_deserialize = "    fn deserialize_slice<T: AsRef<[u8]>>(\n        buf: &mut T,\n    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {\n        use std::io::Cursor;\n        use integer_encoding::VarIntReader;\n        let mut buf = Cursor::new(buf);\n        let packet_id: u32 = buf.read_varint()?;\n        match packet_id {\n".to_string();
            let mut impl_serialize =
                "    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {\n        use integer_encoding::VarIntWriter;\n        let mut res = vec![];\n        res.write_varint(self.packet_id())?;\n        match self {\n"
            .to_string();
            for packet in packets.as_array().unwrap() {
                let packet = packet.as_object().unwrap();
                let packet_name = packet["name"].as_str().unwrap();
                enum_str.push_str(&format!("    {packet_name} {{"));

                let packet_fields = packet["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|field| {
                        let field = field.as_object().unwrap();
                        (
                            field["name"].as_str().unwrap(),
                            field["type"].as_str().unwrap(),
                        )
                    })
                    .collect::<Vec<_>>();

                impl_deserialize.push_str(&format!("            {packet_id} => {{\n"));
                impl_serialize.push_str(&format!("            {enum_name}::{packet_name} {{"));

                let mut first_field = true;
                for (field_name, field_type) in &packet_fields {
                    impl_deserialize.push_str(&format!(
                        "                let {field_name}: {field_type} = {};\n",
                        match *field_type {
                            "String" => "read_packet_string(&mut buf)?".to_string(),
                            "u8" => "byteorder::ReadBytesExt::read_u8(&mut buf)?".to_string(),
                            "u16" => "byteorder::ReadBytesExt::read_u16::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "u32" => "byteorder::ReadBytesExt::read_u32::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "u64" => "byteorder::ReadBytesExt::read_u64::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "i8" => "byteorder::ReadBytesExt::read_i8(&mut buf)?".to_string(),
                            "i16" => "byteorder::ReadBytesExt::read_i16::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "i32" => "byteorder::ReadBytesExt::read_i32::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "i64" => "byteorder::ReadBytesExt::read_i64::<byteorder::BigEndian>(&mut buf)?".to_string(),
                            "usize" => "buf.read_varint::<usize>()?".to_string(),
                            _ => format!("todo!(\"{field_type}\")"),
                        },
                    ));
                    if first_field {
                        enum_str.push(' ');
                    } else {
                        enum_str.push_str(", ");
                    }
                    enum_str.push_str(&format!("{field_name}: {field_type}"));
                    first_field = false;
                }
                if !first_field {
                    enum_str.push(' ');
                }
                enum_str.push_str("},\n");

                let cs_packet_fields = packet_fields
                    .iter()
                    .map(|i| i.0)
                    .collect::<Vec<_>>()
                    .join(", ");

                impl_deserialize.push_str(&format!(
                    "                Ok(Self::{packet_name} {{ {} }})\n            }},\n",
                    cs_packet_fields
                ));

                if !packet_fields.is_empty() {
                    impl_serialize.push(' ');
                }
                impl_serialize.push_str(&cs_packet_fields);
                if !packet_fields.is_empty() {
                    impl_serialize.push(' ');
                }
                impl_serialize.push_str("} => {");
                if !packet_fields.is_empty() {
                    impl_serialize.push_str("\n            ");
                }
                for (field_name, field_type) in &packet_fields {
                    impl_serialize.push_str(&format!(
                        "    {};\n            ",
                        match *field_type {
                            "String" => format!("write_packet_string(&mut res, {field_name})?"),
                            "u8" => format!("byteorder::WriteBytesExt::write_u8(&mut res, *{field_name})?"),
                            "u16" => format!("byteorder::WriteBytesExt::write_u16::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "u32" => format!("byteorder::WriteBytesExt::write_u32::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "u64" => format!("byteorder::WriteBytesExt::write_u64::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "i8" => format!("byteorder::WriteBytesExt::write_i8(&mut res, *{field_name})?"),
                            "i16" => format!("byteorder::WriteBytesExt::write_i16::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "i32" => format!("byteorder::WriteBytesExt::write_i32::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "i64" => format!("byteorder::WriteBytesExt::write_i64::<byteorder::BigEndian>(&mut res, *{field_name})?"),
                            "usize" => format!("res.write_varint(*{field_name})?"),
                            _ => format!("todo!(\"{field_type}\")"),
                        },
                    ));
                }
                impl_serialize.push_str("}\n");

                impl_packet_id.push_str(&format!(
                    "            {enum_name}::{packet_name} {{ .. }} => {packet_id},\n"
                ));
                packet_id += 1;
            }
            enum_str.push_str("    Unknown { packet_id: u32 },\n}\n");
            impl_packet_id.push_str(&format!(
                "            {enum_name}::Unknown {{ .. }} => {packet_id},\n        }}\n    }}\n"
            ));
            let debug_receive_string = if DEBUG_PACKETS {
                "\n        println!(\"Receive: {:X?}\", packet);"
            } else {
                ""
            };
            impl_deserialize.push_str(
                &format!("            _ => Ok(Self::Unknown {{ packet_id }})\n        }}\n    }}\n    async fn deserialize(\n        stream: &mut DynamicRead<'_>,\n    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {{\n        use tokio::io::AsyncReadExt;\n        let packet_length: u32 = read_varint(stream).await.unwrap_or(0).try_into()?;\n        let mut packet = vec![0u8; packet_length as usize];\n        stream.read_exact(&mut packet).await?;\n        let mut packet = packet.as_slice();{}\n        Self::deserialize_slice(&mut packet)\n    }}\n",
                debug_receive_string),
            );
            let debug_send_string = if DEBUG_PACKETS {
                "\n        println!(\"Send: {:X?}\", res);"
            } else {
                ""
            };
            impl_serialize.push_str(&format!(
                "            {enum_name}::Unknown {{ .. }} => {{\n                return Err(\"Cannot serialize unknown packet\".into());\n            }}\n        }};{}\n        #[allow(unreachable_code)]\n        prepend_length(&res)\n    }}\n",
                debug_send_string
            ));
            enum_impl.push_str(&impl_packet_id);
            enum_impl.push_str(&impl_deserialize);
            enum_impl.push_str(&impl_serialize);
            enum_impl.push_str("}\n\n");
            codegen.push_str(&enum_str);
            codegen.push_str(&enum_impl);
        }
    }

    fs::write(dest_path, codegen).unwrap();

    println!("cargo:rerun-if-changed=packets.json");
    println!("cargo:rerun-if-changed=build.rs");
}

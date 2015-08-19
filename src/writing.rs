use std::io::{
	Write,
	Result
};

use std::mem::size_of;

use byteorder::{WriteBytesExt, BigEndian};

use shared::{
	CQL_BINARY_PROTOCOL_VERSION,
	Request,
	QueryFlag,
	BatchType,
	BatchFlag,
	BatchQuery,
	BatchQueryKind,
	Column
};


pub trait WriteMessage {
	fn write_message(&mut self, Request) -> Result<()>;
}

impl<W: Write> WriteMessage for W {
fn write_message(&mut self, message: Request) -> Result<()> {
	let mut header = Vec::new();

	try!(WriteBytesExt::write_u8(&mut header, CQL_BINARY_PROTOCOL_VERSION));
	try!(WriteBytesExt::write_u8(&mut header, 0x00));
	try!(WriteBytesExt::write_i16::<BigEndian>(&mut header, 1));
	try!(WriteBytesExt::write_u8(&mut header, message.opcode()));


	let mut buf = Vec::new();

	match message {
			Request::Startup(ref hash_map) => {
				try!(buf.write_u16::<BigEndian>(hash_map.len() as u16));
				for (key, val) in hash_map.iter() {
					try!(buf.write_u16::<BigEndian>(key.len() as u16));
					try!(Write::write(&mut buf, key.as_bytes()));
					try!(buf.write_u16::<BigEndian>(val.len() as u16));
					try!(Write::write(&mut buf, val.as_bytes()));
				}
			}
			Request::Query(ref query, ref consistency) => {
				try!(buf.write_i32::<BigEndian>(query.len() as i32));
				try!(Write::write(&mut buf, query.as_bytes()));
				try!(buf.write_u16::<BigEndian>((*consistency).clone() as u16));
				try!(WriteBytesExt::write_u8(&mut buf, 0 as u8));
			}
			Request::PrmQuery(ref query, ref values, ref consistency) => {
				try!(buf.write_i32::<BigEndian>(query.len() as i32));
				try!(Write::write(&mut buf, query.as_bytes()));
				try!(buf.write_u16::<BigEndian>((*consistency).clone() as u16));

				try!(WriteBytesExt::write_u8(&mut buf, 0 | QueryFlag::Values as u8));


				try!(buf.write_u16::<BigEndian>(values.len() as u16));

				write_values(&mut buf, values);
			}
			Request::Prepare(ref query) => {
				try!(buf.write_i32::<BigEndian>(query.len() as i32));
				try!(Write::write(&mut buf, query.as_bytes()));
			}
			Request::Execute(ref id, ref values, ref consistency) => {
				try!(buf.write_u16::<BigEndian>(id.len() as u16));
				try!(Write::write(&mut buf, id));
				try!(buf.write_u16::<BigEndian>((*consistency).clone() as u16));

				try!(WriteBytesExt::write_u8(&mut buf, 0 | QueryFlag::Values as u8));


				try!(buf.write_u16::<BigEndian>(values.len() as u16));

				write_values(&mut buf, values);
			}
			Request::Batch(ref queries, ref consistency) => {
				try!(WriteBytesExt::write_u8(&mut buf, BatchType::Logged as u8));
				try!(buf.write_u16::<BigEndian>(queries.len() as u16));
				for query in queries.iter() {
					match query {
						&BatchQuery::Simple(ref query) => {
							try!(WriteBytesExt::write_u8(&mut buf, BatchQueryKind::Simple as u8));
							try!(buf.write_i32::<BigEndian>(query.len() as i32));
							try!(Write::write(&mut buf, query.as_bytes()));
							try!(buf.write_u16::<BigEndian>(0 as u16));
						}
						&BatchQuery::SimpleWithParams(ref query, ref values) => {
							try!(WriteBytesExt::write_u8(&mut buf, BatchQueryKind::Simple as u8));
							try!(buf.write_i32::<BigEndian>(query.len() as i32));
							try!(Write::write(&mut buf, query.as_bytes()));

							try!(buf.write_u16::<BigEndian>(values.len() as u16));

							write_values(&mut buf, values);
						}
						&BatchQuery::Prepared(ref id, ref values) => {
							try!(WriteBytesExt::write_u8(&mut buf, BatchQueryKind::Prepared as u8));
							try!(buf.write_u16::<BigEndian>(id.len() as u16));
							try!(Write::write(&mut buf, id));

							try!(buf.write_u16::<BigEndian>(values.len() as u16));

							write_values(&mut buf, values);
						}
					}
				}
				try!(buf.write_u16::<BigEndian>((*consistency).clone() as u16));
				try!(WriteBytesExt::write_u8(&mut buf, BatchFlag::None as u8));
			}
			_ => ()
		}

	try!(self.write(header.as_slice()));
	try!(self.write_u32::<BigEndian>(buf.len() as u32));
	try!(self.write(buf.as_slice()));

	Ok(())
}
}

fn write_values(buf: &mut Vec<u8>, values: &Vec<Column>) -> Result<()> {
	for col in values.iter() {
		match col {
			&Column::None => {}
			&Column::CqlString(ref v) => { // String
				try!(buf.write_i32::<BigEndian>(v.len() as i32));
				try!(Write::write(buf, v.as_bytes()));
			}
			&Column::CqlInt(ref v) => { // i32
				try!(buf.write_i32::<BigEndian>(size_of::<i32>() as i32));
				try!(buf.write_i32::<BigEndian>(*v));
			}
			&Column::CqlBigint(ref v) =>  { // i64
				try!(buf.write_i32::<BigEndian>(size_of::<i64>() as i32));
				try!(buf.write_i64::<BigEndian>(*v));
			}
			&Column::CqlFloat(ref v) => { //	f32
				try!(buf.write_i32::<BigEndian>(size_of::<f32>() as i32));
				try!(buf.write_f32::<BigEndian>(*v));
			}
			&Column::CqlDouble(ref v) => { // f64
				try!(buf.write_i32::<BigEndian>(size_of::<f64>() as i32));
				try!(buf.write_f64::<BigEndian>(*v));
			}
			&Column::CqlTimestamp(ref v) => { //Tm
				let s = (*v).rfc3339().to_string();
				try!(buf.write_i32::<BigEndian>(s.len() as i32));
				try!(Write::write(buf, s.as_bytes()));
			}
				_ => {}
			}
	}
	Ok(())
}
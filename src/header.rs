use super::{Error, ShapeType};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub(crate) const HEADER_SIZE: i32 = 100;
const FILE_CODE: i32 = 9994;
const SIZE_OF_SKIP: usize = std::mem::size_of::<i32>() * 5;

/// struct representing the Header of a shapefile
/// can be retrieved via the reader used to read
//TODO replace  pointmin/max with bbox + z_range
#[derive(Copy, Clone, PartialEq)]
pub struct Header {
    /// Total file length (Header + Shapes) in 16bit word
    pub file_length: i32,
    /// min values of x, y, z for all the shapes
    pub point_min: [f64; 3],
    /// max values of x, y, z for all the shapes
    pub point_max: [f64; 3],
    /// min and max values for the measure dimension
    pub m_range: [f64; 2],
    /// Type of all the shapes in the file
    /// (as mixing shapes is not allowed)
    pub shape_type: ShapeType,
    /// Version of the shapefile specification
    pub version: i32,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            point_min: [0.0; 3],
            point_max: [0.0; 3],
            m_range: [0.0; 2],
            shape_type: ShapeType::NullShape,
            file_length: HEADER_SIZE / 2,
            version: 1000,
        }
    }
}

impl Header {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Header, Error> {
        let file_code = source.read_i32::<BigEndian>()?;

        if file_code != FILE_CODE {
            return Err(Error::InvalidFileCode(file_code));
        }

        let mut skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        source.read_exact(&mut skip)?;

        let file_length_16_bit = source.read_i32::<BigEndian>()?;
        let version = source.read_i32::<LittleEndian>()?;
        let shape_type = ShapeType::read_from(&mut source)?;

        let mut hdr = Header::default();
        hdr.shape_type = shape_type;
        hdr.version = version;
        hdr.file_length = file_length_16_bit;

        hdr.point_min[0] = source.read_f64::<LittleEndian>()?;
        hdr.point_min[1] = source.read_f64::<LittleEndian>()?;

        hdr.point_max[0] = source.read_f64::<LittleEndian>()?;
        hdr.point_max[1] = source.read_f64::<LittleEndian>()?;

        hdr.point_min[2] = source.read_f64::<LittleEndian>()?;
        hdr.point_max[2] = source.read_f64::<LittleEndian>()?;

        hdr.m_range[0] = source.read_f64::<LittleEndian>()?;
        hdr.m_range[1] = source.read_f64::<LittleEndian>()?;

        Ok(hdr)
    }

    pub(crate) fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<BigEndian>(FILE_CODE)?;

        let skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        dest.write_all(&skip)?;

        dest.write_i32::<BigEndian>(self.file_length)?;
        dest.write_i32::<LittleEndian>(self.version)?;
        dest.write_i32::<LittleEndian>(self.shape_type as i32)?;

        dest.write_f64::<LittleEndian>(self.point_min[0])?;
        dest.write_f64::<LittleEndian>(self.point_min[1])?;
        dest.write_f64::<LittleEndian>(self.point_max[0])?;
        dest.write_f64::<LittleEndian>(self.point_max[1])?;

        dest.write_f64::<LittleEndian>(self.point_min[2])?;
        dest.write_f64::<LittleEndian>(self.point_max[2])?;

        dest.write_f64::<LittleEndian>(self.m_range[0])?;
        dest.write_f64::<LittleEndian>(self.m_range[1])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use byteorder::WriteBytesExt;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn wrong_file_code() {
        use std::io::Cursor;

        let mut src = Cursor::new(vec![]);
        src.write_i32::<BigEndian>(42).unwrap();

        src.seek(SeekFrom::Start(0)).unwrap();
        assert!(Header::read_from(&mut src).is_err());
    }
}

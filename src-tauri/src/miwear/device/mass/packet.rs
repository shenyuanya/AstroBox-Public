use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum MassDataType {
    WATCHFACE = 16,
    FIRMWARE = 32,
    NotificationIcon = 50,
    ThirdpartyApp = 64,
}

#[derive(Clone)]
pub struct MassPacket {
    pub data_type: MassDataType,
    pub md5: Vec<u8>,                // 原始文件数据的MD5
    pub length: u32,                 // 原始文件数据的长度
    pub original_file_data: Vec<u8>, // 原始文件数据
}

impl MassPacket {
    pub fn build(original_file_data: Vec<u8>, data_type: MassDataType) -> Result<Self> {
        Ok(MassPacket {
            data_type,
            md5: crate::tools::calc_md5(&original_file_data), // calc_md5 应返回 Vec<u8>
            length: original_file_data.len() as u32,
            original_file_data,
        })
    }

    /// 编码内部数据块并附加其CRC32。
    /// 输出格式: comp_data (1B) | data_type (1B) | md5 (16B) | length (4B LE) | original_file_data (...) | crc32_of_these_five_fields (4B LE)
    pub fn encode_with_crc32(&self) -> Vec<u8> {
        let mut crc_payload_buf = Vec::with_capacity(
            1 + 1 + self.md5.len() + 4 + self.original_file_data.len() + 4, // 预估容量
        );

        // 1. comp_data (固定为 0x00)
        crc_payload_buf.push(0x00);
        // 2. data_type
        crc_payload_buf.push(self.data_type as u8);
        // 3. md5
        crc_payload_buf.extend_from_slice(&self.md5);
        // 4. length (文件数据长度, 小端)
        crc_payload_buf
            .write_u32::<LittleEndian>(self.length)
            .unwrap();
        // 5. original_file_data
        crc_payload_buf.extend_from_slice(&self.original_file_data);

        // ---到此为止的数据用于计算CRC32---

        // 计算 crc_payload_buf 当前内容的 CRC32
        let crc32_bytes_be = crate::tools::calc_crc32_bytes(&crc_payload_buf); // 假设返回大端 Vec<u8>
        let crc32_val =
            u32::from_be_bytes(crc32_bytes_be.try_into().expect("CRC32 should be 4 bytes"));

        // 将 CRC32 (小端) 追加到缓冲区末尾
        crc_payload_buf
            .write_u32::<LittleEndian>(crc32_val)
            .unwrap();

        crc_payload_buf
    }
}

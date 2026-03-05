const SIZE: usize = 1024 * 4;

#[derive(Clone)]
pub struct SimpleBuffer {
    buf: [u8; SIZE * 3],
    pos: usize,
}

impl SimpleBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0u8; SIZE * 3],
            pos: 0,
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        // 1. 数据超大：只保留最后 SIZE 大小的数据
        if data.len() > SIZE {
            self.buf[..SIZE].copy_from_slice(&data[data.len() - SIZE..]);
            self.pos = SIZE;
            return;
        }

        // 2. 容量不足：将最后的有效数据移动到开头
        if data.len() + self.pos > self.buf.len() {
            let keep = SIZE - data.len();
            // 把最后的 keep 个字节移动到 0..keep
            self.buf.copy_within(self.pos - keep..self.pos, 0);
            // 把新数据追加到 keep 之后，填满到 SIZE
            self.buf[keep..SIZE].copy_from_slice(data);
            self.pos = SIZE;
            return;
        }

        // 3. 正常追加：只截取与 data 长度相等的目标切片进行拷贝
        self.buf[self.pos..self.pos + data.len()].copy_from_slice(data);
        self.pos += data.len();
    }

    pub fn get(&self) -> &[u8] {
        if self.pos <= SIZE {
            return &self.buf[..self.pos];
        }
        &self.buf[self.pos - SIZE..self.pos]
    }

    pub fn to_string(&self) -> String {
        let data = self.get();
        let mut start_pos: usize = 0;

        // 注意：这里基于 data 的长度而不是 self.pos
        let limit = data.len().min(1024);
        for i in 0..limit {
            if data[i] == b'\r' && i + 1 < data.len() && data[i + 1] == b'\n' {
                start_pos = i + 2;
                break;
            } else if data[i] == b'\n' {
                start_pos = i + 1;
                break;
            }
        }

        // 直接从获取到的有效 data 切片中生成字符串
        String::from_utf8_lossy(&data[start_pos..]).to_string()
    }
}

// ================= 测试用例 =================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_and_get() {
        let mut buffer = SimpleBuffer::new();
        buffer.push(b"hello");
        assert_eq!(buffer.get(), b"hello");

        buffer.push(b" world");
        assert_eq!(buffer.get(), b"hello world");
    }

    #[test]
    fn test_push_larger_than_size() {
        let mut buffer = SimpleBuffer::new();
        let mut huge_data = vec![0u8; SIZE + 100];
        // 填充一些特征数据用于验证
        huge_data[SIZE + 99] = 42;

        buffer.push(&huge_data);

        let result = buffer.get();
        assert_eq!(result.len(), SIZE); // 只保留了 SIZE 长度
        assert_eq!(result[result.len() - 1], 42); // 验证保留的是最后的部分
    }

    #[test]
    fn test_buffer_shift_when_full() {
        let mut buffer = SimpleBuffer::new();

        // 填充满整个底层数组 (SIZE * 3 = 12288 字节)
        let fill_data = vec![1u8; SIZE];
        buffer.push(&fill_data); // pos = 4096
        buffer.push(&fill_data); // pos = 8192
        buffer.push(&fill_data); // pos = 12288

        assert_eq!(buffer.pos, SIZE * 3);
        assert_eq!(buffer.get().len(), SIZE); // get 依然只能获取最后的 4096 字节

        // 再 push 就会触发移动机制 (Condition 2)
        let new_data = vec![2u8; 10];
        buffer.push(&new_data);

        // 触发 shift 后，pos 会被重置为 SIZE
        assert_eq!(buffer.pos, SIZE);

        let result = buffer.get();
        assert_eq!(result.len(), SIZE);

        // 验证数据正确性：前面的 SIZE - 10 应该是 1，最后的 10 应该是 2
        let keep_len = SIZE - 10;
        assert!(result[..keep_len].iter().all(|&x| x == 1));
        assert!(result[keep_len..].iter().all(|&x| x == 2));
    }

    #[test]
    fn test_to_string_strip_first_line() {
        let mut buffer = SimpleBuffer::new();

        // 测试 \n 截断
        buffer.push(b"Line1\nLine2");
        assert_eq!(buffer.to_string(), "Line2");

        let mut buffer2 = SimpleBuffer::new();
        // 测试 \r\n 截断
        buffer2.push(b"LineA\r\nLineB");
        assert_eq!(buffer2.to_string(), "LineB");

        let mut buffer3 = SimpleBuffer::new();
        // 测试没有换行符的情况
        buffer3.push(b"NoNewLinesHere");
        assert_eq!(buffer3.to_string(), "NoNewLinesHere");
    }

}
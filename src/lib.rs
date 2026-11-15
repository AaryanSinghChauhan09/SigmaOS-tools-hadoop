#![allow(non_snake_case)]
//! Hadoop HDFS Block Manager for SigmaOS
#[derive(Debug, Clone, PartialEq)]
pub struct HdfsBlock {
    pub block_id: u64,
    pub data: Vec<u8>,
    pub checksum: u32,
    pub replicas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HadoopBlockManager {
    pub default_block_size: usize,
    pub replication_factor: usize,
    next_block_id: u64,
}

impl HadoopBlockManager {
    pub fn new(block_size: usize, replication_factor: usize) -> Self {
        Self {
            default_block_size: block_size,
            replication_factor,
            next_block_id: 1,
        }
    }

    fn compute_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &byte| {
            acc.wrapping_mul(31).wrapping_add(byte as u32)
        })
    }

    pub fn split_and_replicate(&mut self, data: &[u8], datanodes: &[String]) -> Vec<HdfsBlock> {
        let mut blocks = Vec::new();
        let chunks = data.chunks(self.default_block_size);

        for chunk in chunks {
            let block_id = self.next_block_id;
            self.next_block_id += 1;

            let checksum = Self::compute_checksum(chunk);
            let mut replicas = Vec::new();
            if !datanodes.is_empty() {
                for i in 0..self.replication_factor {
                    let idx = ((block_id as usize) + i) % datanodes.len();
                    replicas.push(datanodes[idx].clone());
                }
            }

            blocks.push(HdfsBlock {
                block_id,
                data: chunk.to_vec(),
                checksum,
                replicas,
            });
        }

        blocks
    }

    pub fn verify_and_reconstruct(&self, blocks: &[HdfsBlock]) -> Result<Vec<u8>, &'static str> {
        let mut result = Vec::new();
        for block in blocks {
            let actual_checksum = Self::compute_checksum(&block.data);
            if actual_checksum != block.checksum {
                return Err("Corrupted block checksum");
            }
            result.extend_from_slice(&block.data);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hadoop_hdfs_block_split_and_reconstruct_aaa() {
        // Arrange
        let mut manager = HadoopBlockManager::new(4, 2);
        let datanodes = vec![
            "node1".to_string(),
            "node2".to_string(),
            "node3".to_string(),
        ];
        let input_bytes = b"Hello HDFS SigmaOS Data!";

        // Act
        let blocks = manager.split_and_replicate(input_bytes, &datanodes);
        let reconstructed = manager
            .verify_and_reconstruct(&blocks)
            .expect("Reconstruction failed");

        // Assert
        assert_eq!(blocks.len(), 6); // 24 bytes / 4 bytes per block = 6 blocks
        assert_eq!(blocks[0].replicas.len(), 2);
        assert_eq!(reconstructed, input_bytes);
    }
}

use bitmap::Bitmap;
use block_cache::{BlockCache, block_cache_sync_all, get_block_cache};
use block_device::BlockDevice;
use efs::EasyFileSystem;
use layout::{DIRENT_SIZE, DirctoryEntry, DiskInode, DiskInodeType};
use lazy_static::lazy_static;
use spin::Mutex;
use vfs::Inode;

pub mod bitmap;
pub mod block_cache;
pub mod block_device;
pub mod efs;
pub mod layout;
pub mod vfs;

pub const BLOCK_SIZE: usize = 512;

lazy_static! {
    pub static ref ROOT_FS: Mutex<Option<EasyFileSystem>> = Mutex::new(None);
}

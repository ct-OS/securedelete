//! Borrado seguro nativo para SSD en Linux.
//!
//! La sobrescritura lógica no es fiable en SSD porque el controlador del disco
//! reubica los bloques físicos. Los métodos fiables son los comandos nativos
//! del dispositivo:
//!
//! - **SATA**: *ATA Security Erase* — el firmware del disco borra físicamente
//!   todas las celdas, incluidas las reubicadas. Requiere fijar una contraseña
//!   temporal (`ATA SECURITY SET PASSWORD`, 0xF1) y luego emitir
//!   `ATA SECURITY ERASE UNIT` (0xF4). Ambos viajan como `ATA PASS-THROUGH 16`
//!   (0x85) a través del ioctl `SG_IO`.
//! - **NVMe**: *Format NVM* (0x80) con *Secure Erase Setting* = borrado de
//!   datos de usuario, a través del ioctl `NVME_IOCTL_ADMIN_CMD`.
//!
//! Las estructuras y constantes siguen los *headers* del kernel
//! (`include/uapi/linux/sg.h`, `include/linux/ata.h`,
//! `include/uapi/linux/nvme_ioctl.h`, `include/uapi/linux/hdreg.h`).

#![cfg(any(target_os = "linux", doctest))]

use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::AsRawFd;
use std::path::Path;

use libc::{c_int, c_uchar, c_uint, c_ulong, c_ushort, c_void, ioctl};

// ---------------------------------------------------------------------------
// Constantes SG_IO (include/uapi/linux/sg.h)
// ---------------------------------------------------------------------------

const SG_DXFER_NONE: c_int = -1;
const SG_DXFER_TO_DEV: c_int = -2;
#[allow(dead_code)]
const SG_DXFER_FROM_DEV: c_int = 3;

const SG_INTERFACE_ID_ORIG: c_int = b'S' as c_int;

/// `SG_IO = _IOWR('S', 0, struct sg_io_hdr)` → 0x2285.
const SG_IO: c_ulong = 0x2285;

// ---------------------------------------------------------------------------
// Constantes ATA (include/linux/ata.h)
// ---------------------------------------------------------------------------

const ATA_PASS_THRU16: u8 = 0x85;

const ATA_PT_FLAGS_CK_COND: u8 = 0x02;
#[allow(dead_code)]
const ATA_PT_FLAGS_T_DIR_IN: u8 = 0x04;
const ATA_PT_FLAGS_BYT_BLOK: u8 = 0x08;
const ATA_PT_FLAGS_T_LENGTH: u8 = 0x10;

const ATA_PROT_PIO_OUT: u8 = 5;

const ATA_CMD_SECURITY_SET_PASSWORD: u8 = 0xF1;
const ATA_CMD_SECURITY_ERASE_UNIT: u8 = 0xF4;

const ATA_SEC_SET_PASSWORD: u8 = 0x01;
const ATA_SEC_ERASE: u8 = 0x30;
const ATA_SEC_ERASE_ENHANCED: u8 = 0x31;

const ATA_DEVICE_DEFAULT: u8 = 0xA0;

const SECURITY_BLOCK_SIZE: usize = 512;
const PASSWORD_OFFSET: usize = 2;
const PASSWORD_LEN: usize = 32;

// ---------------------------------------------------------------------------
// Constantes NVMe
// ---------------------------------------------------------------------------

/// `NVME_IOCTL_ADMIN_CMD = _IOWR('N', 0x41, struct nvme_admin_cmd [72 bytes])`.
const NVME_IOCTL_ADMIN_CMD: c_ulong = ((3u32 << 30) as c_ulong)
    | (((72u32 & 0x3FFF) << 16) as c_ulong)
    | ((b'N' as u32) << 8) as c_ulong
    | 0x41u32 as c_ulong;

const NVME_ADMIN_FORMAT: u8 = 0x80;
const NVME_ADMIN_SANITIZE: u8 = 0x81;

const NVME_FMT_SES_USER_DATA: u32 = 1;
const NVME_FMT_SES_CRYPTO: u32 = 2;

const NVME_SANACT_BLOCK_ERASE: u32 = 2;
const NVME_SANACT_CRYPTO_ERASE: u32 = 4;

const NVME_NSID_ALL: u32 = 0xFFFF_FFFF;

// ---------------------------------------------------------------------------
// Estructuras C del kernel
// ---------------------------------------------------------------------------

/// `struct sg_io_hdr` (include/uapi/linux/sg.h). 88 bytes en 64-bit.
#[repr(C)]
#[derive(Default)]
struct SgIoHdr {
    pub interface_id: c_int,
    pub dxfer_direction: c_int,
    pub cmd_len: c_uchar,
    pub mx_sb_len: c_uchar,
    pub iovec_count: c_ushort,
    pub dxfer_len: c_uint,
    pub dxferp: *mut c_void,
    pub cmdp: *mut c_uchar,
    pub sbp: *mut c_uchar,
    pub timeout: c_uint,
    pub flags: c_uint,
    pub pack_id: c_int,
    pub usr_ptr: *mut c_void,
    pub status: c_uchar,
    pub masked_status: c_uchar,
    pub msg_status: c_uchar,
    pub sb_len_wr: c_uchar,
    pub host_status: c_ushort,
    pub driver_status: c_ushort,
    pub resid: c_int,
    pub duration: c_uint,
    pub info: c_uint,
}

/// `struct nvme_admin_cmd` (include/uapi/linux/nvme_ioctl.h). 72 bytes en 64-bit.
#[repr(C)]
#[derive(Default, Clone)]
struct NvmeAdminCmd {
    pub opcode: u8,
    pub flags: u8,
    pub rsvd1: u16,
    pub nsid: u32,
    pub cdw2: u32,
    pub cdw3: u32,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
    pub timeout_ms: u32,
    pub result: u32,
    pub metadata: u64,
    pub data_len: u32,
    pub rsvd2: u32,
    pub data: u64,
}

// ---------------------------------------------------------------------------
// Tipos públicos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaKind {
    Rotational,
    SolidState,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SecureEraseReport {
    #[allow(dead_code)]
    pub kind: MediaKind,
    pub backend: &'static str,
}

// ---------------------------------------------------------------------------
// Detección del tipo de medio
// ---------------------------------------------------------------------------

/// `BLKROTATIONAL = _IOR(0x12, 126, unsigned short)` → 0x127E.
const BLKROTATIONAL: c_ulong = 0x127E;

pub fn is_nvme(path: &Path) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return false,
    };
    name.starts_with("nvme") && name.len() > 4 && name.as_bytes()[4].is_ascii_digit()
}

fn nvme_nsid_from_name(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let stripped = name.strip_prefix("nvme")?;
    let after_controller = stripped.split_once('n')?;
    if after_controller.0.is_empty() || !after_controller.0.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let digits: String = after_controller
        .1
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

pub fn media_kind(path: &Path) -> io::Result<MediaKind> {
    if is_nvme(path) {
        return Ok(MediaKind::SolidState);
    }
    let f = OpenOptions::new().read(true).open(path)?;
    let mut val: c_ushort = 0;
    // SAFETY: BLKROTATIONAL takes a pointer to a unsigned short.
    let rc = unsafe { ioctl(f.as_raw_fd(), BLKROTATIONAL, &mut val) };
    if rc < 0 {
        return Ok(MediaKind::Unknown);
    }
    Ok(if val == 0 {
        MediaKind::SolidState
    } else {
        MediaKind::Rotational
    })
}

// ---------------------------------------------------------------------------
// ATA Security Erase
// ---------------------------------------------------------------------------

/// Construye el CDB ATA PASS-THROUGH 16 (16 bytes).
fn build_ata16_cdb(features: u8, command: u8) -> [u8; 16] {
    // Protocolo PIO Data-Out (host -> device) para SET PASSWORD y ERASE UNIT.
    let mut flags = ATA_PT_FLAGS_CK_COND | ATA_PT_FLAGS_T_LENGTH | ATA_PT_FLAGS_BYT_BLOK;
    // T_DIR_IN = 0 (datos al dispositivo); no se establece el bit.
    let _ = &mut flags;
    let mut cdb = [0u8; 16];
    cdb[0] = ATA_PASS_THRU16;
    cdb[1] = ATA_PROT_PIO_OUT & 0x0F;
    cdb[2] = flags;
    cdb[3] = features;
    cdb[4] = 1; // sector_count = 1 bloque de 512 bytes
    cdb[8] = ATA_DEVICE_DEFAULT;
    cdb[9] = command;
    cdb
}

/// Construye el bloque de información de seguridad (512 bytes):
/// byte 0 = nivel (0x00 = usuario, 0x10 = master); bytes 2..34 = contraseña.
fn build_security_block(password: &[u8], master: bool) -> [u8; SECURITY_BLOCK_SIZE] {
    let mut block = [0u8; SECURITY_BLOCK_SIZE];
    block[0] = if master { 0x10 } else { 0x00 };
    let n = password.len().min(PASSWORD_LEN);
    block[PASSWORD_OFFSET..PASSWORD_OFFSET + n].copy_from_slice(&password[..n]);
    block
}

/// Ejecuta `SG_IO` con el CDB y, opcionalmente, un buffer de datos host→dev.
fn run_sg_io(f: &std::fs::File, cdb: &[u8], data: Option<&mut [u8]>) -> io::Result<()> {
    let mut sb = [0u8; 32];
    let (dxferp, dxfer_len, dir) = match data {
        Some(buf) => (buf.as_mut_ptr() as *mut c_void, buf.len(), SG_DXFER_TO_DEV),
        None => (std::ptr::null_mut(), 0, SG_DXFER_NONE),
    };
    let mut hdr = SgIoHdr {
        interface_id: SG_INTERFACE_ID_ORIG,
        dxfer_direction: dir,
        cmd_len: cdb.len() as u8,
        mx_sb_len: sb.len() as u8,
        iovec_count: 0,
        dxfer_len: dxfer_len as c_uint,
        dxferp,
        cmdp: cdb.as_ptr() as *mut c_uchar,
        sbp: sb.as_mut_ptr(),
        timeout: 21_600_000, // 6 h: a security erase can take hours on big HDDs
        flags: 0,
        pack_id: 0,
        usr_ptr: std::ptr::null_mut(),
        status: 0,
        masked_status: 0,
        msg_status: 0,
        sb_len_wr: 0,
        host_status: 0,
        driver_status: 0,
        resid: 0,
        duration: 0,
        info: 0,
    };
    let rc = unsafe { ioctl(f.as_raw_fd(), SG_IO, &mut hdr as *mut SgIoHdr) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    if hdr.host_status != 0 || hdr.driver_status != 0 {
        return Err(io::Error::other(format!(
            "SG_IO host={} driver={}",
            hdr.host_status, hdr.driver_status
        )));
    }
    if hdr.status != 0 {
        return Err(io::Error::other(format!(
            "SG_IO scsi status=0x{:02X}",
            hdr.status
        )));
    }
    Ok(())
}

fn ata_command(f: &std::fs::File, features: u8, command: u8, password: &[u8]) -> io::Result<()> {
    let mut block = build_security_block(password, false);
    let cdb = build_ata16_cdb(features, command);
    run_sg_io(f, &cdb, Some(&mut block))
}

/// Ejecuta el borrado seguro ATA en un dispositivo SATA.
///
/// Procedimiento (igual que `hdparm`): `SET PASSWORD` con una contraseña
/// temporal y luego `SECURITY ERASE UNIT`. La contraseña se borra al terminar.
pub fn ata_secure_erase(path: &Path, enhanced: bool) -> io::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open(path)?;
    let password: &[u8] = b"x";
    ata_command(
        &f,
        ATA_SEC_SET_PASSWORD,
        ATA_CMD_SECURITY_SET_PASSWORD,
        password,
    )?;
    let features = if enhanced {
        ATA_SEC_ERASE_ENHANCED
    } else {
        ATA_SEC_ERASE
    };
    ata_command(&f, features, ATA_CMD_SECURITY_ERASE_UNIT, password)
}

// ---------------------------------------------------------------------------
// NVMe
// ---------------------------------------------------------------------------

pub fn nvme_secure_erase(path: &Path, crypto: bool) -> io::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open(path)?;
    let nsid = nvme_nsid_from_name(path).unwrap_or(NVME_NSID_ALL);
    let ses = if crypto {
        NVME_FMT_SES_CRYPTO
    } else {
        NVME_FMT_SES_USER_DATA
    };
    let mut cmd = NvmeAdminCmd {
        opcode: NVME_ADMIN_FORMAT,
        nsid,
        cdw10: ses << 8,
        timeout_ms: 3_600_000, // 1 h: large namespaces can format slowly
        ..Default::default()
    };
    let rc = unsafe { ioctl(f.as_raw_fd(), NVME_IOCTL_ADMIN_CMD, &mut cmd) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// SANITIZE con Block Erase (`crypto=false`) o Crypto Erase (`crypto=true`).
#[allow(dead_code)]
pub fn nvme_sanitize(path: &Path, crypto: bool) -> io::Result<()> {
    let f = OpenOptions::new().read(true).write(true).open(path)?;
    let sanact = if crypto {
        NVME_SANACT_CRYPTO_ERASE
    } else {
        NVME_SANACT_BLOCK_ERASE
    };
    let mut cmd = NvmeAdminCmd {
        opcode: NVME_ADMIN_SANITIZE,
        nsid: 0,
        cdw10: sanact & 0xFF,
        timeout_ms: 0,
        ..Default::default()
    };
    let rc = unsafe { ioctl(f.as_raw_fd(), NVME_IOCTL_ADMIN_CMD, &mut cmd) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Backend de alto nivel: detecta el medio y aplica la estrategia nativa.
/// `enhanced` selecciona la variante criptográfica cuando el dispositivo la
/// soporta (ATA enhanced security erase; NVMe crypto erase).
pub fn secure_erase_device(path: &Path, enhanced: bool) -> io::Result<SecureEraseReport> {
    if is_nvme(path) {
        nvme_secure_erase(path, enhanced)?;
        return Ok(SecureEraseReport {
            kind: MediaKind::SolidState,
            backend: if enhanced {
                "NVMe Format NVM (crypto erase)"
            } else {
                "NVMe Format NVM (Secure Erase: user data erase)"
            },
        });
    }
    let kind = media_kind(path)?;
    match kind {
        MediaKind::SolidState => {
            ata_secure_erase(path, enhanced)?;
            Ok(SecureEraseReport {
                kind,
                backend: if enhanced {
                    "ATA Security Erase (enhanced)"
                } else {
                    "ATA Security Erase (normal)"
                },
            })
        }
        MediaKind::Rotational => Err(io::Error::other(
            "rotational device: use overwrite wiping (-d -m <method>) instead of --secure-erase",
        )),
        MediaKind::Unknown => Err(io::Error::other(
            "cannot determine device kind; refusing native erase — use overwrite wiping",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sg_io_hdr_layout() {
        let size = std::mem::size_of::<SgIoHdr>();
        assert!(size == 88 || size == 96, "sg_io_hdr size={size}");
        assert_eq!(std::mem::offset_of!(SgIoHdr, interface_id), 0);
        assert_eq!(std::mem::offset_of!(SgIoHdr, dxfer_direction), 4);
        assert_eq!(std::mem::offset_of!(SgIoHdr, cmd_len), 8);
        assert_eq!(std::mem::offset_of!(SgIoHdr, dxfer_len), 12);
        assert_eq!(std::mem::offset_of!(SgIoHdr, dxferp), 16);
        assert_eq!(std::mem::offset_of!(SgIoHdr, cmdp), 24);
        assert_eq!(std::mem::offset_of!(SgIoHdr, status), 64);
    }

    #[test]
    fn ata16_cdb_for_erase_unit() {
        let cdb = build_ata16_cdb(ATA_SEC_ERASE, ATA_CMD_SECURITY_ERASE_UNIT);
        assert_eq!(cdb.len(), 16);
        assert_eq!(cdb[0], 0x85);
        assert_eq!(cdb[1], ATA_PROT_PIO_OUT & 0x0F);
        assert_eq!(cdb[3], ATA_SEC_ERASE);
        assert_eq!(cdb[4], 1);
        assert_eq!(cdb[8], ATA_DEVICE_DEFAULT);
        assert_eq!(cdb[9], ATA_CMD_SECURITY_ERASE_UNIT);
    }

    #[test]
    fn security_block_layout() {
        let b = build_security_block(b"hunter2", false);
        assert_eq!(b.len(), 512);
        assert_eq!(b[0], 0x00);
        assert_eq!(&b[2..2 + 7], b"hunter2");
        assert_eq!(b[9], 0);
        let m = build_security_block(b"p", true);
        assert_eq!(m[0], 0x10);
        assert_eq!(m[2], b'p');
    }

    #[test]
    fn nvme_admin_cmd_layout() {
        let s = std::mem::size_of::<NvmeAdminCmd>();
        assert_eq!(s, 72, "nvme_admin_cmd size={s}");
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, opcode), 0);
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, nsid), 4);
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, cdw10), 16);
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, timeout_ms), 40);
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, metadata), 48);
        assert_eq!(std::mem::offset_of!(NvmeAdminCmd, data), 64);
    }

    #[test]
    fn nvme_nsid_parsing() {
        assert_eq!(nvme_nsid_from_name(Path::new("/dev/nvme0n1")), Some(1));
        assert_eq!(nvme_nsid_from_name(Path::new("/dev/nvme3n2")), Some(2));
        assert_eq!(nvme_nsid_from_name(Path::new("/dev/nvme0")), None);
        assert_eq!(nvme_nsid_from_name(Path::new("/dev/sda")), None);
    }

    #[test]
    fn is_nvme_detection() {
        assert!(is_nvme(Path::new("/dev/nvme0n1")));
        assert!(is_nvme(Path::new("/dev/nvme5n3")));
        assert!(!is_nvme(Path::new("/dev/sda")));
        assert!(!is_nvme(Path::new("/dev/nvme")));
    }

    #[test]
    fn nvme_format_cdw10_ses() {
        // SES en bits 8-9 de cdw10.
        let cmd = NvmeAdminCmd {
            cdw10: NVME_FMT_SES_USER_DATA << 8,
            ..Default::default()
        };
        assert_eq!(cmd.cdw10 & 0x300, 0x100);
        let cmd2 = NvmeAdminCmd {
            cdw10: NVME_FMT_SES_CRYPTO << 8,
            ..cmd
        };
        assert_eq!(cmd2.cdw10 & 0x300, 0x200);
    }

    #[test]
    fn blockrotational_ioctl_value() {
        assert_eq!(BLKROTATIONAL, 0x127Eu64 as c_ulong);
    }

    #[test]
    fn nvme_ioctl_admin_cmd_value() {
        let expected: u64 = (3u64 << 30) | ((72u64 & 0x3FFF) << 16) | (('N' as u64) << 8) | 0x41;
        assert_eq!(NVME_IOCTL_ADMIN_CMD, expected as c_ulong);
    }
}

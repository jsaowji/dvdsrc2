use std::{
    fs::File,
    io::{Cursor, Seek, SeekFrom},
    path::Path,
    time::Instant,
};

use byteorder::{ReadBytesExt, BE};
use std::io::Read;

use crate::{
    bindings::dvdread::*,
    dvdio::cache_seek_reader::CacheSeekReader,
    dvdio::dvd_reader::DvdReader,
    dvdio::proper_dvd_reader::ProperDvdReader,
    index::{
        AudioStreamPackets, Frame, FrameType, Gop, IndexManager, IndexedVts, KnowUnit, Stream, Vobu,
    },
    mpeg2video::{
        extension_header, group_of_pictures_header, picture_header, sequence_header, Extension,
    },
    mpegps::{pes, start_code, ReadToVec},
    open_dvd,
};

pub fn get_index_vts(a: &str, vts: u8) -> IndexedVts {
    let idm = IndexManager::new();
    let dd = idm.get_dir(a);
    //let fz = dd.join(format!("{vts}.json"));
    let fz = dd.join(format!("{vts}.bin"));

    if fz.exists() {
        if let Ok(e) = bincode::decode_from_std_read(
            &mut File::open(&fz).unwrap(),
            bincode::config::standard(),
        ) {
            return e;
        }

        //if let Ok(e) = bson::from_reader(File::open(fz).unwrap()) {
        //    return e;
        //}

        //        if let Ok(e) = serde_json::from_reader(File::open(&fz).unwrap()) {
        //            return e;
        //        }
    }
    do_index_vts(a, vts as _).unwrap();
    return get_index_vts(a, vts);
}

pub fn do_index_vts<P: AsRef<Path>>(stra: P, i: i32) -> Result<(), Box<dyn std::error::Error>> {
    let dvd = open_dvd(stra.as_ref().to_str().unwrap()).unwrap();

    let idm = IndexManager::new();

    let dir = idm.get_dir(stra);
    std::fs::create_dir_all(&dir).unwrap();

    let value = do_index(dvd, i).unwrap();

    let cnts = bincode::encode_to_vec(&value, bincode::config::standard()).unwrap();
    //serde_json::to_writer(File::create(dir.join(format!("{i}.json"))).unwrap(), &value).unwrap();

    std::fs::write(dir.join(format!("{i}.bin")), cnts).unwrap();
    unsafe {
        DVDClose(dvd);
    }

    Ok(())
}
/*
pub fn do_index_all<P: AsRef<Path>>(stra: P) -> Result<u8, Box<dyn std::error::Error>> {
    unsafe {
        let dvd = open_dvd(stra.as_ref().to_str().unwrap()).unwrap();

        let ifo0 = ifoOpen(dvd, 0);
        assert!(!ifo0.is_null());

        let idm = IndexManager::new();

        let dir = idm.get_dir(stra);
        std::fs::create_dir_all(&dir).unwrap();

        for i in 1..(*(*ifo0).vts_atrt).nr_of_vtss as i32 + 1 {
            eprintln!("vts {}", i);
            let value = do_index(dvd, i).unwrap();

            //let jesen = File::create(dir.join(format!("{}.json", i)))?;
            //let jesen = BufWriter::with_capacity(1024 * 1024 * 10, jesen);

            //serde_json::to_writer(jesen, &value).unwrap();
            //let cnts = bson::to_vec(&value)?;
            let cnts = bincode::encode_to_vec(&value, bincode::config::standard()).unwrap();

            std::fs::write(dir.join(format!("{i}.bin")), cnts).unwrap();
        }

        let vtss = (*(*ifo0).vts_atrt).nr_of_vtss as u8;
        ifoClose(ifo0);
        //DVDCLOSE
        Ok(vtss)
    }
}
*/
pub struct OpenDvdBlockReader {
    pub reader: CacheSeekReader<ProperDvdReader>,
    pub maxsize: u64,
}

impl OpenDvdBlockReader {
    pub fn new(dvd: *mut dvd_reader_s, vts: i32) -> OpenDvdBlockReader {
        unsafe {
            let file = DVDOpenFile(dvd, vts, dvd_read_domain_t_DVD_READ_TITLE_VOBS);
            assert!(!file.is_null());

            // let ifo = ifoOpen(dvd, vts);
            //assert!(!ifo.is_null());

            let dvd_reader = DvdReader::new(file);
            let maxsize = dvd_reader.block_cnt * 2048;
            let b = ProperDvdReader::new(dvd_reader);

            let b = CacheSeekReader::new(b);
            //ifoClose(ifo);
            return OpenDvdBlockReader { reader: b, maxsize };
        }
    }
}

fn do_index(dvd: *mut dvd_reader_s, vts: i32) -> Result<IndexedVts, std::io::Error> {
    let r = OpenDvdBlockReader::new(dvd, vts);

    let maxsize = r.maxsize;
    let mut b = r.reader;

    let mut video_buffer: Vec<u8> = Vec::with_capacity(2048 * 1000);

    let mut vobu_n = 0;

    let mut frames_seen = 0;

    let mut last_print = Instant::now();
    let mut last_frames = 0;

    let mut scratch = [0u8; 2048];

    let mut current_vobu = Vobu::default();
    let mut vobus = Vec::new();

    let finalize_vobu = |current_vobu: &mut Vobu,
                         vobus: &mut Vec<Vobu>,
                         video_data: &mut Vec<u8>,
                         frames_seen: &mut usize| {
        current_vobu.mpeg2_video_size = video_data.len() as _;
        if video_data.len() > 0 {
            let mut gape = Gopaliser::doit(&video_data).unwrap();

            for g in &mut gape.0 {
                *frames_seen += g.frames.len();

                let fcnt = g.frames.len();

                let fclone = g.frames.clone();
                for a in &mut g.frames {
                    let currenttr = a.temporal_reference;
                    if currenttr >= fcnt as u8 {
                        let maxi = fclone
                            .iter()
                            .filter(|e| e.temporal_reference != currenttr)
                            .map(|e| e.temporal_reference)
                            .max();
                        a.temporal_reference = maxi.unwrap() + 1;
                        eprintln!("FOUND BAD TEMPORAL REFERENCE TRYING TO FIX\nWILL ONLY WORK IF ITS THE ONE CASE I SAW");
                    }
                }
                let mut sort_frames = g.frames.clone();

                sort_frames.sort_by(|a, b| {
                    a.temporal_reference
                        .partial_cmp(&b.temporal_reference)
                        .unwrap()
                });

                assert_eq!(sort_frames[0].temporal_reference, 0);
                assert_eq!(
                    sort_frames[sort_frames.len() - 1].temporal_reference,
                    sort_frames.len() as u8 - 1
                );
            }
            video_data.clear();
            current_vobu.gops = gape.0;
            current_vobu.mpeg2_has_seq_end = gape.1;
        }

        //flush current vobu
        vobus.push(current_vobu.clone());
        *current_vobu = Default::default();
    };

    loop {
        let crnt = b.seek(SeekFrom::Current(0))?;
        //eprintln!("{:.02} \r",100.0 * crnt as f32 / maxsize as f32);
        if crnt == maxsize {
            break;
        }
        //dbg!(b.seek(SeekFrom::Current(0))?,maxsize);

        assert_eq!(start_code(&mut b)?, 0xBA);
        b.read_exact(&mut scratch[0..10])?;
        loop {
            let nxt = start_code(&mut b)?;
            let sz = b.read_u16::<BE>()? as usize;
            //if crnt >= (6.5 * 1024.0 * 1024.0 * 1024.0) as _{
            //    println!("{}   {:X}",crnt,nxt);
            //}

            match nxt {
                0xBF => {
                    let mut v = b.read_to_cursor(sz)?;
                    match v.read_u8()? {
                        0 => {
                            if vobu_n >= 1 {
                                if (Instant::now() - last_print).as_millis() > 1000 {
                                    eprintln!(
                                        "{}fps {:.02}\r",
                                        frames_seen - last_frames,
                                        100.0 * crnt as f32 / maxsize as f32
                                    );
                                    last_frames = frames_seen;
                                    last_print = Instant::now();
                                }
                                finalize_vobu(
                                    &mut current_vobu,
                                    &mut vobus,
                                    &mut video_buffer,
                                    &mut frames_seen,
                                );
                            }

                            vobu_n += 1;

                            //let mut pci = std::mem::zeroed();
                            //dvdread::navRead_PCI(&mut pci, v.into_inner()[1..].as_mut_ptr());
                            //let data = v.into_inner()[1..];
                            v.seek(SeekFrom::Start(1 + 0xC)).unwrap();

                            current_vobu.first_ptm = v.read_u32::<BE>().unwrap();
                            //current_vobu.first_ptm = pci.pci_gi.vobu_s_ptm;
                            current_vobu.sector_start = (crnt / 2048) as u32;
                        }
                        1 => {
                            //let mut dsi = std::mem::zeroed();
                            //dvdread::navRead_DSI(&mut dsi, v.into_inner()[1..].as_mut_ptr());

                            v.seek(SeekFrom::Start(1 + 0x18)).unwrap();
                            current_vobu.vobcell.vob = v.read_u16::<BE>().unwrap();
                            _ = v.read_u8().unwrap();
                            current_vobu.vobcell.cell = v.read_u8().unwrap();
                            //current_vobu.vobcell.vob = dsi.dsi_gi.vobu_vob_idn;
                            //current_vobu.vobcell.cell = dsi.dsi_gi.vobu_c_idn;
                        }
                        _ => unreachable!(),
                    }
                }
                0xBE | 0xBB => {
                    b.read_exact(&mut scratch[0..sz as usize])?;
                }
                0xBD => {
                    let pes = pes(&mut b, sz)?;
                    let veca = pes.inner.into_inner();

                    let id = veca[0];

                    //                    if crnt >= (6.5 * 1024.0 * 1024.0 * 1024.0) as _{
                    //                        dbg!(id);
                    //                    }
                    let is_ac3 = id >= 0x80 && id <= 0x87;
                    let is_lpcm = id >= 0xA0 && id <= 0xA7;
                    if is_ac3 || is_lpcm {
                        let audio_header = &veca[1..];

                        let frame_cnt = audio_header[0];
                        let first_acc_unit =
                            ((audio_header[1] as u32) << 8) + audio_header[2] as u32;

                        let data_bytes_left = &audio_header[3..];

                        if current_vobu.streams.iter().find(|e| e.id == id).is_none() {
                            current_vobu.streams.push(Stream {
                                id,
                                first_ptm: pes.pts.unwrap(),
                                packets: AudioStreamPackets::default(),
                            });
                        }

                        let strm = current_vobu
                            .streams
                            .iter_mut()
                            .find(|e| e.id == id)
                            .unwrap();
                        strm.packets.abs_packet_cnt += 1;
                        strm.packets.starting_frames += frame_cnt as u32;
                        let pre_total_bytes = strm.packets.total_bytes;
                        if is_ac3 {
                            strm.packets.total_bytes += data_bytes_left.len() as u32;
                        } else if is_lpcm {
                            strm.packets.total_bytes += data_bytes_left.len() as u32;
                        }
                        if first_acc_unit != 0 {
                            strm.packets.know_units.push(KnowUnit {
                                offset: pre_total_bytes + first_acc_unit as u32 - 1,
                                frame_cnt: frame_cnt as u8,
                                pts: pes.pts.unwrap(),
                            });
                        }
                    }
                }
                0xE0 => {
                    let pes = pes(&mut b, sz)?;

                    video_buffer.extend_from_slice(&pes.inner.into_inner());
                }
                e => {
                    panic!("{:X}", e);
                }
            }
            if b.seek(SeekFrom::Current(0))? % 2048 == 0 {
                break;
            }
        }
    }

    finalize_vobu(
        &mut current_vobu,
        &mut vobus,
        &mut video_buffer,
        &mut frames_seen,
    );
    //let ifo = ifoOpen(dvd, vts as _);
    //let admap = *(*ifo).vts_vobu_admap;
    //let vobu_cnt = (admap.last_byte + 1 - VOBU_ADMAP_SIZE) as isize / 4;
    //assert_eq!(vobu_cnt as usize, vobus.len());
    //for kk in 0..vobu_cnt {
    //    let mm = *admap.vobu_start_sectors.offset(kk);
    //    assert_eq!(vobus[kk as usize].sector_start, mm);
    //}
    //ifoClose(ifo);
    for (i, kk) in parse_vobu_admap(&get_ifo_file(dvd, vts as _))
        .into_iter()
        .enumerate()
    {
        assert_eq!(vobus[i].sector_start, kk);
    }

    return Ok(IndexedVts {
        vobus,
        total_size_blocks: maxsize as u32 / 2048,
    });
}

struct Gopaliser {}

impl Gopaliser {
    pub fn doit(video_data: &[u8]) -> Result<(Vec<Gop>, bool), std::io::Error> {
        let mut b = Cursor::new(video_data.clone());

        let mut has_seq_end = false;

        let mut ii = 0;

        let mut gops = Vec::new();
        let mut current_gop = Gop::default();

        let mut statey = 0;

        assert_eq!(video_data[0..4], [0x00, 0x00, 0x01, 0xB3]);

        loop {
            //let st = start_code(&mut b).unwrap();

            if ii + 3 >= video_data.len() {
                break;
            }
            if video_data[ii] == 0 && video_data[ii + 1] == 0 && video_data[ii + 2] == 1 {
                let st = video_data[ii + 3];
                b.seek(SeekFrom::Start(ii as u64 + 4))?;
                match st {
                    0x00 => {
                        assert_eq!((statey - 3) % 2, 0);
                        statey += 1;

                        let pics = picture_header(&mut b)?;

                        current_gop.frames.push(Frame::default());
                        let ll = current_gop.frames.len();
                        let cf = &mut current_gop.frames[ll - 1];

                        cf.temporal_reference = pics.temporal_reference as u8;
                        cf.real_temporal_reference = pics.temporal_reference as u8;
                        cf.frametype = match pics.picture_type {
                            1 => FrameType::I,
                            2 => FrameType::P,
                            3 => FrameType::B,
                            _ => unreachable!(),
                        };
                    }
                    0xB8 => {
                        assert_eq!(statey, 2);
                        statey = 3;
                        let gop = group_of_pictures_header(&mut b)?;

                        current_gop.closed = gop.closed;
                        current_gop.frames.clear();

                        assert_eq!(gop.broken, false);
                    }
                    0xB3 => {
                        if statey != 0 {
                            gops.push(current_gop.clone());
                        }
                        //assert_eq!(statey, 0);
                        statey = 1;
                        let seq = sequence_header(&mut b)?;
                        current_gop.sequence_header = seq;
                    }
                    0xB5 => {
                        if let Some(e) = extension_header(&mut b)? {
                            match e {
                                Extension::Sequence {
                                    progressive_sequence,
                                } => {
                                    current_gop.prog_sequence = progressive_sequence;

                                    assert_eq!(statey, 1);
                                    statey = 2;
                                }
                                Extension::Picture { tff, rff, prog } => {
                                    assert_eq!((statey - 3) % 2, 1);
                                    statey += 1;

                                    let ll = current_gop.frames.len();
                                    let cf = &mut current_gop.frames[ll - 1];
                                    cf.tff = tff;
                                    cf.rff = rff;
                                    cf.prog = prog;
                                }
                            }
                        }
                    }
                    0xB2 => {
                        //user data
                    }
                    0xB7 => {
                        has_seq_end = true;
                        eprintln!("seq end");
                    }
                    e => {
                        if e >= 0x01 && e <= 0xAF {
                        } else {
                            eprintln!("{:X}", e);
                        }
                    }
                }
                ii += 1;
            } else {
                ii += 1;
            }
        }
        gops.push(current_gop.clone());
        Ok((gops, has_seq_end))
    }
}

pub fn get_ifo_file(dvd: *mut dvd_reader_t, ifo: u8) -> Vec<u8> {
    unsafe {
        let mut stat = std::mem::zeroed();

        DVDFileStat(
            dvd,
            ifo as _,
            dvd_read_domain_t_DVD_READ_INFO_FILE,
            &mut stat,
        );
        let ifofilesize = stat.size;
        let file = DVDOpenFile(dvd, ifo as _, dvd_read_domain_t_DVD_READ_INFO_FILE);

        let mut buffer = vec![0u8; ifofilesize as usize];

        let rdrd = DVDReadBytes(file, buffer.as_mut_ptr() as _, ifofilesize as _);
        assert_eq!(rdrd as usize, buffer.len());

        DVDCloseFile(file);
        return buffer;
    }
}

fn parse_vobu_admap(buffer: &[u8]) -> Vec<u32> {
    let mut cur = Cursor::new(buffer);
    cur.seek(SeekFrom::Start(0x00E4)).unwrap();
    let skp = cur.read_u32::<BE>().unwrap() as u64 * 2048;
    cur.seek(SeekFrom::Start(skp)).unwrap();

    let end = cur.read_u32::<BE>().unwrap();

    let cnt = (end + 1 - 4) / 4;
    (0..cnt)
        .into_iter()
        .map(|_| cur.read_u32::<BE>().unwrap())
        .collect()
}

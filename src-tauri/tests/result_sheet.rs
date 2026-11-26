//! The patient's result sheet, made from the sample session in D:\SmartFoot_Contoh (book sample data,
//! override with SMARTFOOT_SAMPLE). The patient folder is copied to a temporary folder first, so the
//! sample is never changed. Skipped with a note when the sample is missing.

use smartfoot_app_lib::{pdf, session};
use std::path::{Path, PathBuf};

fn sample() -> PathBuf {
    std::env::var_os("SMARTFOOT_SAMPLE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"D:\SmartFoot_Contoh"))
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let t = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), t).unwrap();
        }
    }
}

#[test]
fn sheet_from_the_sample_session() {
    let src = sample();
    let Some((sd, _)) = session::list_sesi(&src).into_iter().next() else {
        println!("sample data not found at {}, skipping", src.display());
        return;
    };
    let tmp = std::env::temp_dir().join(format!("sf_sheet_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let dst = tmp.join(sd.file_name().unwrap());
    copy_dir(&sd, &dst);
    let pats = session::list_patients(&dst);
    let (dir, p) = pats.first().expect("a sample patient").clone();
    assert!(p.completed);
    // no sheet yet (sample made before 0.3.0): ensure_sheet makes it
    let s = session::Settings { penyelenggara: "Tim SmartFoot".into(), ..session::Settings::default() };
    assert_eq!(pdf::ensure_sheet(&dir, &s), None);
    let f = pdf::find_sheet(&dir).expect("sheet made");
    let name = f.file_name().unwrap().to_string_lossy().to_string();
    assert!(name.starts_with(&format!("Hasil skrining kaki - {}", p.nama)), "{name}");
    let bytes = std::fs::read(&f).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.len() < 3_000_000, "sheet is {} bytes, too big for WhatsApp comfort", bytes.len());
    let t0 = std::fs::metadata(&f).unwrap().modified().unwrap();
    // nothing changed: not made again
    assert_eq!(pdf::ensure_sheet(&dir, &s), None);
    assert_eq!(std::fs::metadata(&f).unwrap().modified().unwrap(), t0);
    // a change the patient reads changes the fingerprint and makes the sheet again
    let fp = pdf::fingerprint_of(&dir, &session::read_patient(&dir).unwrap());
    session::set_notes(&dir, &session::Notes { umum: "Kulit tumit kering.".into(), ..Default::default() }, Some("Rina")).unwrap();
    let p2 = session::read_patient(&dir).unwrap();
    assert_ne!(pdf::fingerprint_of(&dir, &p2), fp);
    assert_eq!(pdf::ensure_sheet(&dir, &s), None);
    // a header change remakes the file but is not a change for the patient
    let fp2 = pdf::fingerprint_of(&dir, &p2);
    let s2 = session::Settings { kontak: "0812-0000-0000".into(), ..s.clone() };
    assert_eq!(pdf::ensure_sheet(&dir, &s2), None);
    assert_eq!(pdf::fingerprint_of(&dir, &session::read_patient(&dir).unwrap()), fp2);
    // keep one sheet for eyes-on checking
    let keep = std::env::temp_dir().join("sf_sheet_sample.pdf");
    std::fs::copy(pdf::find_sheet(&dir).unwrap(), &keep).unwrap();
    println!("sheet: {} ({} bytes), copy at {}", name, bytes.len(), keep.display());
    let msg = pdf::wa_message(&p2, pdf::sesi_of(&dir).as_ref(), &s);
    let first = p2.nama.split_whitespace().next().unwrap();
    assert!(msg.starts_with(&format!("Halo {} {first} 🙏", if p2.kelamin == "Perempuan" { "Ibu" } else { "Bapak" })), "{msg}");
    assert!(msg.contains(&format!("{} – Tim SmartFoot", p2.operator)), "{msg}");
    assert!(!msg.contains('{'), "a placeholder was left: {msg}");
    let _ = std::fs::remove_dir_all(&tmp);
}

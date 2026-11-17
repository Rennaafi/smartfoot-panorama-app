//! INLOW 60-second diabetic foot screen, in the scored form the client uses
//! ("New ISi SKOR PENGKAJIAN KAKI (2).xlsx", received 17 Nov 2026).
//!
//! The published original (Inlow / Sibbald, Adv Skin Wound Care 2012) is a yes/no screen. The client
//! scores each item with points and sums them per foot into risk classes A-E. Everything about that
//! scoring lives in this file, so a change the client asks for is a change here only.
//!
//! Choices made where the client's sheet disagrees with itself (listed for the client in
//! `SCORING_NOTES`, shown in the exported Excel):
//!  * Pulse is counted ONCE. The client's total formula adds the pulse column twice (`Z11+Z11`).
//!  * Pulse is scored 0 = present / 1 = absent, as on the client's form (the input sheet's legend for
//!    that column was a copy of the deformity legend).
//!  * "Risiko urgen" = total of 26 or more, as on the input sheet (the form repeats "20 - 25").
//!  * Nails are scored 1-3 as the client wrote, so a healthy foot never totals 0.
//!  * The four neuropathy questions score 2 when ANY answer is yes (the original Inlow rule); the
//!    client's wording "ya, untuk semua" leaves a partial yes undefined.

use serde::{Deserialize, Serialize};

pub struct Item {
    pub key: &'static str,
    /// Group heading in the client's sheet.
    pub group: &'static str,
    pub label: &'static str,
    /// (points, text) exactly as in the client's form.
    pub options: &'static [(u8, &'static str)],
}

pub const ITEMS: [Item; 12] = [
    Item { key: "kulit", group: "LIHAT", label: "Kulit", options: &[(0, "Utuh atau sehat"), (1, "Kering dengan jamur atau terdapat kalus tipis"), (2, "Kalus tebal"), (3, "Luka terbuka atau riwayat luka")] },
    Item { key: "kuku", group: "LIHAT", label: "Kuku", options: &[(1, "Terawat"), (2, "Tidak terawat"), (3, "Tebal, rusak, kasar")] },
    Item { key: "deformitas", group: "LIHAT", label: "Deformitas", options: &[(0, "Tidak ada deformitas"), (2, "Deformitas ringan"), (4, "Deformitas mayor")] },
    Item { key: "alas_kaki", group: "LIHAT", label: "Alas kaki", options: &[(0, "Sesuai kebutuhan"), (1, "Tidak sesuai"), (2, "Menyebabkan trauma")] },
    Item { key: "suhu_dingin", group: "SENTUH", label: "Suhu dingin", options: &[(0, "Hangat"), (1, "Dingin")] },
    Item { key: "suhu_panas", group: "SENTUH", label: "Suhu panas", options: &[(0, "Hangat"), (1, "Panas")] },
    Item { key: "rom", group: "SENTUH", label: "Gerakan / ROM", options: &[(0, "Full range to hallux"), (1, "Hallux limitus"), (2, "Hallux rigidus"), (3, "Hallux amputation")] },
    Item { key: "monofilamen", group: "SENSASI", label: "Sensori / monofilamen", options: &[(0, "10 titik terdeteksi"), (2, "7-9 titik terdeteksi"), (4, "0-6 titik terdeteksi")] },
    Item { key: "tanya", group: "SENSASI", label: "4 pertanyaan", options: &[(0, "Tidak, untuk semua pertanyaan"), (2, "Ya, untuk salah satu atau lebih")] },
    Item { key: "pulsasi", group: "SENSASI", label: "Pulsasi nadi dorsalis pedis dan tibia posterior", options: &[(0, "Ada"), (1, "Tidak ada")] },
    Item { key: "kemerahan", group: "SENSASI", label: "Kemerahan bergantung posisi kaki", options: &[(0, "Tidak ada"), (1, "Ada")] },
    Item { key: "eritema", group: "SENSASI", label: "Eritema", options: &[(0, "Tidak ada"), (1, "Ada")] },
];

/// The four questions behind the "tanya" item (client's form, wording kept).
pub const QUESTIONS: [&str; 4] = [
    "Apakah kakimu mati rasa?",
    "Apakah kaki Anda pernah tergelitik?",
    "Apakah kaki Anda pernah terasa seperti terbakar?",
    "Apakah kaki Anda pernah merasa seperti serangga merayap di atasnya?",
];

pub const SCORING_NOTES: &str = "Catatan skoring: pulsasi dihitung satu kali (rumus di lembar asli menjumlahkannya dua kali); \
pulsasi 0 = ada, 1 = tidak ada; risiko urgen = total 26 atau lebih; kuku dinilai 1-3 sesuai lembar asli; \
4 pertanyaan bernilai 2 bila salah satu atau lebih dijawab ya.";

/// Scores of one foot: item key -> points. Missing key = not filled in yet.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct FootScores {
    #[serde(default)]
    pub items: std::collections::BTreeMap<String, u8>,
    /// Answers to the four questions, if the nurse ticked them (None = not asked).
    #[serde(default)]
    pub questions: [Option<bool>; 4],
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Inlow {
    #[serde(default)]
    pub kanan: FootScores,
    #[serde(default)]
    pub kiri: FootScores,
}

pub fn item(key: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|i| i.key == key)
}

/// Keep only known items with allowed point values; derive "tanya" from the question answers when
/// all four were answered.
pub fn clean(mut f: FootScores) -> Result<FootScores, String> {
    for (k, v) in &f.items {
        let it = item(k).ok_or_else(|| format!("BAD_INLOW: unknown item {k}"))?;
        if !it.options.iter().any(|o| o.0 == *v) {
            return Err(format!("BAD_INLOW: {v} is not a score for {k}"));
        }
    }
    if f.questions.iter().all(|q| q.is_some()) {
        let any_yes = f.questions.iter().any(|q| *q == Some(true));
        f.items.insert("tanya".into(), if any_yes { 2 } else { 0 });
    }
    Ok(f)
}

impl FootScores {
    pub fn complete(&self) -> bool {
        ITEMS.iter().all(|i| self.items.contains_key(i.key))
    }
    /// Total when every item is filled in.
    pub fn total(&self) -> Option<u32> {
        self.complete().then(|| self.items.values().map(|&v| v as u32).sum())
    }
}

pub struct RiskClass {
    pub letter: char,
    pub name: &'static str,
    pub indication: &'static str,
}

/// Client's table: 0-6 A, 7-12 B, 13-19 C, 20-25 D, 26+ E.
pub fn risk_class(total: u32) -> RiskClass {
    match total {
        0..=6 => RiskClass { letter: 'A', name: "Risiko sangat rendah", indication: "Sensasi + Vaskularisasi -" },
        7..=12 => RiskClass { letter: 'B', name: "Risiko rendah", indication: "Sensasi -, ada/tidak ada deformitas" },
        13..=19 => RiskClass { letter: 'C', name: "Risiko sedang", indication: "PAD + dan/ada deformitas, sensasi -" },
        20..=25 => RiskClass { letter: 'D', name: "Risiko tinggi", indication: "Diabetes dengan riwayat luka" },
        _ => RiskClass { letter: 'E', name: "Risiko urgen", indication: "Luka terbuka, Charcot foot, Gangrene" },
    }
}

/// The client's MANAJEMEN table, word for word (only obvious spelling slips fixed).
/// Mapping agreed 17 Nov 2026 (to be confirmed with the client): A+B -> RENDAH, C -> SEDANG,
/// D -> TINGGI, E -> ADA LUKA, taken from the worse foot.
pub fn management(letter: char) -> (&'static str, &'static [&'static str]) {
    match letter {
        'A' | 'B' => ("Risiko rendah", &[
            "Penggunaan alas kaki yang tepat",
            "Perawatan kaki/footspa 1x sebulan",
            "Kontrol gula darah ke dokter spesialis",
            "Kontrol kebutuhan nutrisi ke ahli gizi",
            "Lakukan olahraga teratur",
            "Kontrol kebutuhan ke rehabilitasi medik",
        ]),
        'C' => ("Risiko sedang", &[
            "Penggunaan alas kaki yang tepat",
            "Perawatan kaki/footspa 2 minggu 1x",
            "Terapi akupuntur untuk neuropati",
            "Kontrol gula darah ke dokter spesialis",
            "Kontrol kebutuhan nutrisi ke ahli gizi",
            "Lakukan olahraga teratur",
            "Kontrol kebutuhan ke rehabilitasi medik",
        ]),
        'D' => ("Risiko tinggi", &[
            "Penggunaan alas kaki yang tepat",
            "Perawatan kaki/footspa seminggu sekali",
            "Terapi akupuntur untuk neuropati",
            "Kontrol gula darah ke dokter spesialis",
            "Kontrol kebutuhan nutrisi ke ahli gizi",
            "Kontrol kebutuhan ke rehabilitasi medik",
        ]),
        _ => ("Ada luka", &[
            "Rujuk ke perawat spesialis luka",
            "Setelah sembuh lakukan pengkajian ulang",
            "Kontrol gula darah ke dokter spesialis",
            "Lakukan olahraga teratur sesuai kemampuan",
        ]),
    }
}

/// The worse (later) class letter of the two feet, if any foot is complete.
pub fn worse_letter(inl: &Inlow) -> Option<char> {
    [inl.kanan.total(), inl.kiri.total()].into_iter().flatten().map(|t| risk_class(t).letter).max()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full(points: [u8; 12]) -> FootScores {
        let mut f = FootScores::default();
        for (i, p) in ITEMS.iter().zip(points) {
            f.items.insert(i.key.into(), p);
        }
        f
    }

    #[test]
    fn totals_and_classes_follow_the_client_table() {
        // client's example row 12 (Perempuan 41): only nails = 1 on both feet -> total 1 -> A
        let f = full([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(f.total(), Some(1));
        assert_eq!(risk_class(1).letter, 'A');
        for (t, l) in [(0, 'A'), (6, 'A'), (7, 'B'), (12, 'B'), (13, 'C'), (19, 'C'), (20, 'D'), (25, 'D'), (26, 'E'), (40, 'E')] {
            assert_eq!(risk_class(t).letter, l, "total {t}");
        }
        // the highest possible total is 26 with pulse counted once
        let max = full([3, 3, 4, 2, 1, 1, 3, 4, 2, 1, 1, 1]);
        assert_eq!(max.total(), Some(26));
        assert_eq!(risk_class(26).letter, 'E');
    }

    #[test]
    fn incomplete_foot_has_no_total() {
        let mut f = full([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        f.items.remove("eritema");
        assert!(!f.complete() && f.total().is_none());
    }

    #[test]
    fn bad_scores_are_refused() {
        let mut f = FootScores::default();
        f.items.insert("kuku".into(), 0); // nails start at 1
        assert!(clean(f).unwrap_err().starts_with("BAD_INLOW"));
        let mut f = FootScores::default();
        f.items.insert("deformitas".into(), 1); // 0, 2 or 4 only
        assert!(clean(f).is_err());
        let mut f = FootScores::default();
        f.items.insert("nonsense".into(), 0);
        assert!(clean(f).is_err());
    }

    #[test]
    fn questions_set_the_tanya_item() {
        let mut f = FootScores { questions: [Some(false); 4], ..Default::default() };
        assert_eq!(clean(f.clone()).unwrap().items["tanya"], 0);
        f.questions[2] = Some(true);
        assert_eq!(clean(f.clone()).unwrap().items["tanya"], 2);
        f.questions[0] = None; // not all answered: tanya left as the nurse set it
        assert!(!clean(f).unwrap().items.contains_key("tanya"));
    }

    #[test]
    fn worse_foot_picks_the_advice() {
        let inl = Inlow { kanan: full([0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), kiri: full([2, 2, 2, 1, 1, 0, 1, 2, 2, 0, 0, 0]) };
        assert_eq!(inl.kiri.total(), Some(13));
        assert_eq!(worse_letter(&inl), Some('C'));
        assert_eq!(management('C').0, "Risiko sedang");
        assert_eq!(management('B').0, management('A').0);
        assert_eq!(management('E').1[0], "Rujuk ke perawat spesialis luka");
        assert_eq!(worse_letter(&Inlow::default()), None);
    }
}

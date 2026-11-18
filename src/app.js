// SmartFoot front end. All hardware, file and image work happens in Rust; this file only draws
// screens and calls the commands in src-tauri/src/lib.rs.
//
// Flow (v2): a screening session ("sesi" = one day, one place, one laptop) holds patients. Each
// patient goes Data pasien -> Kaki kiri -> (berbalik) -> Kaki kanan -> INLOW -> Catatan -> Selesai.
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

const T = {
  id: {
    steps: ["Data pasien", "Kaki kiri", "Kaki kanan", "INLOW", "Catatan", "Selesai"], team: "⚙ Settings",
    // session
    home: "Mulai sesi", homeSub: "Satu sesi = satu hari, di satu tempat, dengan satu laptop.",
    when: "Tanggal dan jam", whenNote: "diisi otomatis", place: "Tempat / kegiatan", placePh: "mis. Posyandu Melati, Beji",
    op: "Operator", opPh: "Nama operator", opPick: "Pernah dipakai di laptop ini:", goSesi: "Mulai sesi →", startingSesi: "Membuka sesi…",
    e_PLACE: "Isi tempat atau kegiatan.", e_OPNAME: "Isi nama operator.",
    stale: "Sesi sebelumnya belum diakhiri", staleSub: "Sesi ini dari hari lain. Akhiri dulu, lalu mulai sesi baru untuk hari ini.",
    endOld: "Akhiri sesi itu", patients: "pasien",
    hubOp: "Operator", chOp: "Ganti operator", chOpT: "Ganti operator", chOpSub: "Pasien berikutnya dicatat atas nama operator baru.", ok: "OK",
    pause: "Jeda", endS: "Akhiri sesi", endSure: "Yakin? Tekan lagi untuk mengakhiri",
    newP: "+ Pasien baru", list: "Pasien sesi ini", none: "Belum ada pasien.",
    cNo: "No.", cName: "Nama", cOp: "Operator", cScans: "Pindaian", cInlow: "INLOW", cConc: "Kesimpulan",
    inlowOk: "lengkap", inlowNo: "INLOW belum lengkap", open: "Buka", cancelled: "dibatalkan", running: "belum selesai",
    unf: "Pasien belum selesai", unfSub: "Aplikasi tertutup di tengah pemeriksaan pasien ini. Lanjutkan dari langkah berikutnya.",
    resume: "Lanjutkan pasien", resuming: "Membuka…",
    paused: "Sesi dijeda", pausedSub: "Layar dikunci selama jeda. Tidak ada pasien yang bisa dimulai.", since: "Sejak", unpause: "Lanjutkan sesi →",
    ended: "Sesi diakhiri", endedSub: "Rekap Excel sesi ini sudah dibuat di folder data.",
    // patient form
    pform: "Data pasien", pformSub: "Nama, usia, dan jenis kelamin wajib diisi.", pEdit: "Ubah data pasien",
    nama: "Nama", age: "Usia", ageU: "tahun", sex: "Jenis kelamin", f: "Perempuan", m: "Laki-laki",
    td: "Tekanan darah", tdPh: "120/80", bb: "Berat badan", bbU: "kg", opt: "opsional", req: "wajib",
    pos: "Posisi pemindaian", posB: "Berdiri", posD: "Duduk", posNote: "Bawaan = posisi pasien sebelumnya.",
    goScan: "Mulai pindai kaki kiri →", starting: "Memeriksa scanner…", save: "Simpan", back: "← Kembali",
    e_C_NAME: "Isi nama pasien.", e_C_AGE: "Usia harus bilangan bulat 0–120.", e_C_SEX: "Pilih jenis kelamin.",
    e_C_TD: "Tekanan darah ditulis seperti 120/80.", e_C_BB: "Berat badan harus 20–250 kg.",
    // scan
    scan: "Pindai", left: "Kaki kiri", right: "Kaki kanan",
    subStandL: "Pasien berdiri dengan kedua kaki di platform, kaki kiri di atas kaca dan tumit menempel pada penahan. Berdiri seimbang dan diam sampai pemindaian selesai.",
    subStandR: "Kaki kanan di atas kaca, tumit menempel pada penahan. Berdiri seimbang dan diam sampai pemindaian selesai.",
    subSitL: "Pasien duduk. Letakkan kaki kiri di atas kaca, tumit pada penahan. Diam sampai pemindaian selesai.",
    subSitR: "Letakkan kaki kanan di atas kaca, tumit pada penahan. Diam sampai pemindaian selesai.",
    doScan: "Mulai pindai", scanning: "Memindai… jangan gerakkan kaki", ready: "Siap", retry: "Coba lagi",
    turn: "Silakan berbalik", turnSub: "Minta pasien berbalik badan menghadap sisi lain platform, lalu letakkan kaki kanan di atas kaca dengan tumit pada penahan.",
    turnGo: "Pindai kaki kanan →",
    review: "Tinjau hasil", reviewSub: "Garis biru = tepi kaki yang terdeteksi. Pastikan sesuai bentuk kaki.",
    chkFound: "Kaki terdeteksi", chkInside: "Kaki penuh di dalam area",
    wNoFoot: "Kaki tidak terdeteksi. Pastikan kaki menempel di kaca, lalu ulangi.",
    wEdge: "Kaki menyentuh tepi gambar. Geser kaki menjauhi tepi, lalu ulangi.",
    wOdd: "Bentuknya tidak seperti kaki (terlalu kecil, besar, atau memanjang). Pastikan yang dipindai adalah kaki.",
    wTwo: "Ada benda kedua di kaca. Hanya satu yang diberi garis. Singkirkan benda lain, lalu ulangi.",
    accept: "Terima", acceptAnyway: "Tetap terima", retake: "Ulangi", saving: "Menyimpan…",
    cancelP: "Batalkan pasien", cancelSure: "Yakin? Tekan lagi untuk batal",
    // point check
    chk: "Periksa titik", chkSub: "Seret titik yang belum tepat. Tahan titik untuk melihat kaca pembesar.",
    chkClarke: "Titik oranye = sudut Clarke: tepi dalam kaki depan, tepi dalam tumit, dan titik terdalam lengkung.",
    chkAxis: "Titik putih = sumbu kaki untuk Arch Index: pusat tumit dan ujung jari ke-2.",
    lFore: "depan", lHeel: "tumit", lArch: "lengkung", lHc: "pusat tumit", lToe: "jari ke-2",
    clarke: "Sudut Clarke", guessK: "Tebakan aplikasi dari peta kontak", guessP: "Tebakan kasar aplikasi (peta kontak tidak cukup jelas)",
    noLabel: "Label datar / normal / tinggi baru diberikan setelah titik diperiksa.",
    aiNow: "Arch Index (garis tepi)", aiLater: "dihitung ulang setelah konfirmasi bila titik putih digeser",
    moved: "titik digeser", reset: "Kembalikan otomatis", confirm: "✓ Titik sudah benar", skip: "Lewati",
    skipQ: "Alasan melewati pemeriksaan:", skipR: { "antrean panjang": "Antrean panjang", "gambar kurang jelas": "Gambar kurang jelas", "lainnya": "Lainnya" },
    roughWarn: "Titik oranye belum digeser dari tebakan kasar aplikasi. Tebakan kasar hampir selalu salah di titik lengkung. Periksa lagi, atau tekan sekali lagi jika titik memang sudah benar.", roughOk: "✓ Ya, titik sudah benar",
    skipNote: "Kaki ini akan tercatat \"belum diperiksa\" dan bisa diperiksa nanti di Detail pengukuran.",
    lyOutline: "Garis tepi", lyAI: "Indeks lengkung", lyClarke: "Sudut Clarke", lyContact: "Peta kontak", lyMarks: "Tanda manual",
    contactCap: "Hijau terang = tertekan kuat, hijau redup = tertekan ringan.",
    lbl: { datar: "datar", normal: "normal", tinggi: "tinggi" }, okChk: "diperiksa", okSkip: "dilewati",
    stNone: "belum diperiksa", ptCheck: "Pemeriksaan titik", hist: "pemeriksaan sebelumnya tersimpan di riwayat", recheck: "Periksa ulang",
    onlyUnchk: "Hanya yang belum diperiksa", allChecked: "Semua kaki sudah diperiksa.",
    remT: "Masih ada kaki yang belum diperiksa", remSub: "Untuk kaki di bawah ini, sudut Clarke tercatat sebagai perkiraan aplikasi tanpa label datar / normal / tinggi. Periksa sekarang, atau akhiri sesi saja; kaki ini tetap bisa diperiksa nanti di Detail pengukuran.",
    remEnd: "Tetap akhiri sesi",
    // INLOW
    inlow: "Formulir INLOW", inlowSub: "Ketuk satu pilihan per kaki. Ketuk lagi untuk menghapus. Boleh disimpan walau belum lengkap.",
    kanan: "Kaki kanan", kiri: "Kaki kiri", filled: "terisi", total: "Total", qOpen: "4 pertanyaan", yes: "Ya", no: "Tidak",
    qNote: "Jika keempat pertanyaan dijawab, nilai di atas diisi otomatis.",
    toNotes: "Simpan, lanjut ke Catatan →",
    // notes
    notes: "Catatan", notesSub: "Catatan per kaki dan untuk pasien. Semua opsional.",
    nKiri: "Catatan kaki kiri", nKanan: "Catatan kaki kanan", nUmum: "Catatan pasien",
    mark: "Tandai secara manual", markOff: "Selesai menandai", markSub: "Ketuk gambar untuk memberi tanda. Ketuk tanda untuk menghapusnya.",
    mLabel: "Jenis tanda", mKalus: "Kalus", mLuka: "Luka", mLain: "Lainnya", mLainPh: "tulis jenis tanda", marks: "tanda", noScan: "Belum dipindai",
    finish: "Simpan dan selesai ✓", finishing: "Menyimpan…", toScan: "Simpan, lanjut pindai →", toList: "← Daftar pasien",
    // finished
    done: "Pasien selesai", doneSub: "Semua data pasien tersimpan.", saved: "Tersimpan di", conc: "Kesimpulan",
    nextP: "Pasien berikutnya →", toHub: "Daftar pasien", resBtn: "Detail pengukuran →",
    // errors from Rust ("CODE: detail")
    e_BAD_IMAGE: "Hasil pemindaian rusak atau terlalu kecil. Ulangi pemindaian.",
    dmg: "Ada folder pasien yang rusak atau tidak lengkap", dmgHint: "Jangan dihapus. Beri tahu tim.",
    e_EMPTY_NAME: "Nama kosong.", e_LONG_NAME: "Nama terlalu panjang (maksimal 60 huruf).", e_EMPTY_AGE: "Isi usia.",
    e_BAD_AGE: "Usia harus bilangan bulat 0–120.", e_BAD_SEX: "Pilih jenis kelamin.", e_BAD_TD: "Tekanan darah ditulis seperti 120/80.",
    e_BAD_BB: "Berat badan harus 20–250 kg.", e_BAD_POSISI: "Pilih posisi berdiri atau duduk.",
    e_BAD_DIR: "Folder harus berupa alamat lengkap, misalnya D:\\SmartFoot_Data.",
    e_SESI_OPEN: "Masih ada sesi yang terbuka.", e_SESI_ENDED: "Sesi ini sudah diakhiri.", e_SESI_PAUSED: "Sesi sedang dijeda.", e_NO_SESI: "Tidak ada sesi yang terbuka.",
    e_PATIENT_OPEN: "Masih ada pasien yang belum selesai. Selesaikan atau batalkan dulu.", e_NO_PATIENT: "Folder pasien tidak ditemukan.",
    e_ID_USED: "Folder pasien dengan nomor dan nama ini sudah ada.", e_BAD_INLOW: "Nilai INLOW tidak dikenal.", e_BAD_MARK: "Ada tanda di luar gambar.",
    e_FILE_OPEN: "File Excel rekap tidak bisa diperbarui karena sedang dibuka. Tutup Excel; file diperbarui lagi pada penyimpanan berikutnya", e_PICTURE: "Gambar analisis tidak bisa dibuat",
    e_TIMEOUT: "Scanner tidak merespons. Cabut dan colok kembali kabel USB, lalu ulangi.",
    e_NOT_FOUND: "Scanner tidak ditemukan. Periksa kabel USB dan pastikan scanner menyala.",
    e_LOCKED: "Scan tidak bisa dimulai. Pastikan kunci transportasi scanner (LOCK) terbuka dan scanner terpasang lengkap (kaca bawaan scanner harus terpasang), lalu ulangi.",
    e_BUSY: "Scanner sedang sibuk. Tunggu beberapa detik lalu coba lagi.",
    e_DISK_FULL: "Disk penuh. Kosongkan ruang (satu pemindaian butuh sekitar 35 MB) atau ganti folder data di Settings.",
    e_SCAN_FAILED: "Pemindaian gagal", e_NOT_RESUMABLE: "Pasien ini sudah selesai atau dibatalkan.", e_BAD_STATION: "Kode laptop harus 1–4 huruf atau angka.",
    e_BACKUP_SAME: "Folder salinan harus berbeda dari folder data (bukan di dalamnya, bukan induknya).", e_BACKUP_FAILED: "Folder salinan tidak bisa dipakai", e_BACKUP_OFF: "Folder salinan belum diatur.",
    // settings, scanner, backup, data folder, first run
    set: "Pengaturan", setSub: "Pengaturan: folder data, scanner, salinan cadangan, dan kalibrasi skala.",
    dataf: "Folder data", saved2: "Tersimpan ✓", station: "Kode laptop", stationNote: "Awalan ID internal pasien, unik per laptop (1–4 huruf/angka). Jangan sama dengan laptop lain.",
    scanner: "Scanner", auto: "Otomatis (LiDE 300)", refresh: "↻ Refresh",
    cal: "Kalibrasi skala", calNote: "Dikalibrasi 13 Nov 2026 dengan penggaris baja dan plastik di atas akrilik (kasing akhir), dicek ulang 18 Nov 2026 dengan lembar kalibrasi di platform berdiri. Memanjang (arah gerak scanner): 300,7 dpi = 11,84 piksel per mm. Melintang: 299,5 dpi = 11,79 piksel per mm. Nilai ini dicatat di setiap file pasien.",
    calCheck: "Cek ulang setelah scanner dipindah, spacer diganti, atau akrilik diganti: pindai lembar kalibrasi atau penggaris baja, lalu beri tahu tim jika jarak 1 mm tidak lagi 11,84 piksel.",
    savedBtn: "✓ Tersimpan", browse: "Pilih folder…", bkDone: "Perhatian: salinan kedua belum berhasil. Data hanya ada di satu disk sampai folder salinan bisa diakses lagi.",
    scChecking: "Memeriksa scanner…", scReady: "Scanner siap", scMissing: "Scanner tidak terdeteksi", scAttn: "Scanner perlu dicek",
    scMissingMsg: "Cabut lalu colok kembali kabel USB scanner, dan pastikan scanner menyala.",
    scNullMsg: "Windows memasang driver kosong (Null Driver) sehingga scanner tidak bisa dipakai. Cabut lalu colok kembali kabel USB; jika tetap begini, hubungi tim.",
    scPanel: "Pemeriksaan scanner", scState: "Status", scFound: "Perangkat terdeteksi", scNone: "tidak ada", scRoute: "Jalur yang dipakai", scNullRow: "Null Driver",
    scDriverless: "Driverless (disarankan)", scCanon: "Driver Canon", scOther: "Lainnya", scNo: "tidak ada", scYes: "ADA, cabut dan colok kembali",
    scTest: "Tes scanner", scTesting: "Menguji… (sampai ±25 detik)", scTestNote: "Memindai strip kecil tanpa kaki di kaca. Membuktikan scanner benar-benar bisa memindai dan kunci LOCK terbuka.",
    scLast: "Tes terakhir", scNever: "belum ada", scPass: "berhasil", scFail: "gagal",
    bkLabel: "Salinan data kedua", bkNote: "Folder di disk, USB atau cloud lain. Setiap penyimpanan disalin otomatis. Kosongkan untuk mematikan.",
    bkNow: "Cadangkan sekarang", bkNowBusy: "Menyalin…", bkOff: "Salinan kedua belum diatur (data hanya ada di satu disk).", bkOk: "Salinan terakhir berhasil", bkFiles: "file", bkNever: "Belum ada salinan sejak aplikasi dibuka.",
    bkGone: "Folder salinan tidak bisa diakses (USB terlepas?). Data baru belum tersalin.", bkFail: "Salinan terakhir gagal",
    ddSaved: "Data disimpan di", ddChange: "Ubah", ddBad: "Folder data tidak bisa diakses",
    ddBadHint: "Colokkan kembali flash disk atau drive-nya, atau pilih folder lain lewat Ubah. Pemindaian belum bisa dimulai.",
    setupT: "Selamat datang di SmartFoot", setupSub: "Sebelum mulai, tentukan di mana hasil pemindaian disimpan di laptop ini.",
    setupData: "Folder data", required: "wajib", setupDataNote: "Semua sesi dan pemindaian disimpan di sini, di disk laptop atau di flash disk.",
    setupBk: "Salinan kedua (opsional)", setupBkNote: "Folder di disk lain, USB, atau cloud. Kosongkan untuk melewati; data lalu hanya ada di satu disk.",
    setupCode: "Kode laptop", setupGo: "Simpan dan mulai →", setupSaving: "Menyimpan…",
    // measurement details
    res: "Detail pengukuran", resSub: "Semua angka dari pemindaian.", resSess: "Sesi", resPat: "Pasien",
    adjust: "Sesuaikan titik acuan", adjustOff: "Selesai menyesuaikan",
    resNone: "Belum ada pasien yang dipindai.", resPts: "Seret dua titik putih: pusat tumit (bawah) dan ujung jari ke-2 (atas), lalu lepas. Angka dihitung ulang.",
    resCorr: "Titik dikoreksi", yes2: "ya", no2: "tidak", resAuto: "Kembalikan otomatis",
    resUncal: "Belum dikalibrasi: angka memakai 300 dpi nominal, jangan dipakai sebagai hasil akhir.", resNoMeas: "Pemindaian ini tidak bisa diukur (kaki tidak ditemukan).",
    resGap: "Dasar jari dari celah antar jari", resProp: "Dasar jari diperkirakan 82% panjang (jari menempel)",
  },
  en: {
    steps: ["Patient", "Left foot", "Right foot", "INLOW", "Notes", "Done"], team: "⚙ Settings",
    home: "Start a session", homeSub: "One session = one day, at one place, on one laptop.",
    when: "Date and time", whenNote: "filled in automatically", place: "Place / activity", placePh: "e.g. Posyandu Melati, Beji",
    op: "Operator", opPh: "Operator name", opPick: "Used before on this laptop:", goSesi: "Start session →", startingSesi: "Opening the session…",
    e_PLACE: "Enter the place or activity.", e_OPNAME: "Enter the operator's name.",
    stale: "The previous session was not ended", staleSub: "This session belongs to another day. End it first, then start a new session for today.",
    endOld: "End that session", patients: "patients",
    hubOp: "Operator", chOp: "Change operator", chOpT: "Change operator", chOpSub: "The next patients are recorded under the new operator.", ok: "OK",
    pause: "Break", endS: "End session", endSure: "Sure? Press again to end",
    newP: "+ New patient", list: "Patients in this session", none: "No patients yet.",
    cNo: "No.", cName: "Name", cOp: "Operator", cScans: "Scans", cInlow: "INLOW", cConc: "Conclusion",
    inlowOk: "complete", inlowNo: "INLOW incomplete", open: "Open", cancelled: "cancelled", running: "not finished",
    unf: "Unfinished patient", unfSub: "The app closed in the middle of this patient. Continue from the next step.",
    resume: "Resume patient", resuming: "Opening…",
    paused: "Session on a break", pausedSub: "The screen is locked during the break. No patient can be started.", since: "Since", unpause: "Resume session →",
    ended: "Session ended", endedSub: "The session's Excel summary has been written to the data folder.",
    pform: "Patient details", pformSub: "Name, age and sex are required.", pEdit: "Edit patient details",
    nama: "Name", age: "Age", ageU: "years", sex: "Sex", f: "Female", m: "Male",
    td: "Blood pressure", tdPh: "120/80", bb: "Body weight", bbU: "kg", opt: "optional", req: "required",
    pos: "Scan posture", posB: "Standing", posD: "Seated", posNote: "Default = the previous patient's posture.",
    goScan: "Start scanning the left foot →", starting: "Checking the scanner…", save: "Save", back: "← Back",
    e_C_NAME: "Enter the patient's name.", e_C_AGE: "The age must be a whole number from 0 to 120.", e_C_SEX: "Choose the sex.",
    e_C_TD: "Write the blood pressure like 120/80.", e_C_BB: "The body weight must be 20–250 kg.",
    scan: "Scan", left: "Left foot", right: "Right foot",
    subStandL: "The patient stands with both feet on the platform, the left foot on the glass and the heel against the stop. Stand evenly and keep still until the scan ends.",
    subStandR: "Right foot on the glass, heel against the stop. Stand evenly and keep still until the scan ends.",
    subSitL: "The patient sits. Place the left foot on the glass, heel against the stop. Keep still until the scan ends.",
    subSitR: "Place the right foot on the glass, heel against the stop. Keep still until the scan ends.",
    doScan: "Start scan", scanning: "Scanning… keep the foot still", ready: "Ready", retry: "Try again",
    turn: "Please turn around", turnSub: "Ask the patient to turn around to face the other side of the platform, then place the right foot on the glass with the heel against the stop.",
    turnGo: "Scan the right foot →",
    review: "Review result", reviewSub: "Blue line = detected foot edge. Make sure it follows the foot.",
    chkFound: "Foot detected", chkInside: "Whole foot inside the area",
    wNoFoot: "No foot detected. Make sure the foot rests on the glass, then retake.",
    wEdge: "The foot touches the image edge. Move it away from the edge, then retake.",
    wOdd: "The shape does not look like a foot (too small, too large or too elongated). Make sure a foot was scanned.",
    wTwo: "There is a second object on the glass. Only one is outlined. Remove the other object, then retake.",
    accept: "Accept", acceptAnyway: "Accept anyway", retake: "Retake", saving: "Saving…",
    cancelP: "Cancel patient", cancelSure: "Sure? Press again to cancel",
    chk: "Check the points", chkSub: "Drag any point that is not right. Hold a point to see the magnifier.",
    chkClarke: "Orange points = Clarke's angle: inner edge of the forefoot, inner edge of the heel, and the deepest point of the arch.",
    chkAxis: "White points = foot axis for the Arch Index: heel centre and 2nd-toe tip.",
    lFore: "forefoot", lHeel: "heel", lArch: "arch", lHc: "heel centre", lToe: "2nd toe",
    clarke: "Clarke's angle", guessK: "App's guess from the contact map", guessP: "App's rough guess (the contact map was not clear enough)",
    noLabel: "The flat / normal / high label is only given after the points are checked.",
    aiNow: "Arch Index (outline)", aiLater: "recalculated after confirming if a white point was moved",
    moved: "points moved", reset: "Reset to automatic", confirm: "✓ Points are right", skip: "Skip",
    skipQ: "Reason for skipping the check:", skipR: { "antrean panjang": "Long queue", "gambar kurang jelas": "Image not clear", "lainnya": "Other" },
    roughWarn: "The orange points have not been moved from the app's rough guess. The rough guess is nearly always wrong at the arch point. Check again, or press once more if the points really are right.", roughOk: "✓ Yes, the points are right",
    skipNote: "This foot is recorded as \"not checked\" and can be checked later in Measurement details.",
    lyOutline: "Outline", lyAI: "Arch index", lyClarke: "Clarke's angle", lyContact: "Contact map", lyMarks: "Manual marks",
    contactCap: "Bright green = pressed firmly, dim green = pressed lightly.",
    lbl: { datar: "flat", normal: "normal", tinggi: "high" }, okChk: "checked", okSkip: "skipped",
    stNone: "not checked", ptCheck: "Point check", hist: "earlier checks kept in the history", recheck: "Check again",
    onlyUnchk: "Only not checked", allChecked: "Every foot has been checked.",
    remT: "Some feet have not been checked", remSub: "For the feet below, Clarke's angle is recorded as the app's estimate without a flat / normal / high label. Check them now, or end the session anyway; they can still be checked later in Measurement details.",
    remEnd: "End the session anyway",
    inlow: "INLOW form", inlowSub: "Tap one choice per foot. Tap again to clear it. It may be saved incomplete.",
    kanan: "Right foot", kiri: "Left foot", filled: "filled in", total: "Total", qOpen: "4 questions", yes: "Yes", no: "No",
    qNote: "When all four questions are answered, the value above is filled in automatically.",
    toNotes: "Save, go to Notes →",
    notes: "Notes", notesSub: "Notes per foot and for the patient. All optional.",
    nKiri: "Left foot note", nKanan: "Right foot note", nUmum: "Patient note",
    mark: "Mark by hand", markOff: "Done marking", markSub: "Tap the image to place a mark. Tap a mark to remove it.",
    mLabel: "Mark type", mKalus: "Callus", mLuka: "Wound", mLain: "Other", mLainPh: "type of mark", marks: "marks", noScan: "Not scanned yet",
    finish: "Save and finish ✓", finishing: "Saving…", toScan: "Save, continue scanning →", toList: "← Patient list",
    done: "Patient done", doneSub: "All patient data is saved.", saved: "Saved to", conc: "Conclusion",
    nextP: "Next patient →", toHub: "Patient list", resBtn: "Measurement details →",
    e_BAD_IMAGE: "The scan is damaged or too small. Retake the scan.",
    dmg: "Some patient folders are damaged or incomplete", dmgHint: "Do not delete them. Tell the team.",
    e_EMPTY_NAME: "The name is empty.", e_LONG_NAME: "The name is too long (60 letters at most).", e_EMPTY_AGE: "Enter the age.",
    e_BAD_AGE: "The age must be a whole number from 0 to 120.", e_BAD_SEX: "Choose the sex.", e_BAD_TD: "Write the blood pressure like 120/80.",
    e_BAD_BB: "The body weight must be 20–250 kg.", e_BAD_POSISI: "Choose standing or seated.",
    e_BAD_DIR: "The folder must be a full path, for example D:\\SmartFoot_Data.",
    e_SESI_OPEN: "A session is still open.", e_SESI_ENDED: "This session has ended.", e_SESI_PAUSED: "The session is on a break.", e_NO_SESI: "No session is open.",
    e_PATIENT_OPEN: "A patient is not finished yet. Finish or cancel that patient first.", e_NO_PATIENT: "The patient folder was not found.",
    e_ID_USED: "A patient folder with this number and name already exists.", e_BAD_INLOW: "Unknown INLOW value.", e_BAD_MARK: "A mark is outside the image.",
    e_FILE_OPEN: "The Excel summary could not be updated because it is open. Close Excel; the file is updated again at the next save", e_PICTURE: "An analysis picture could not be made",
    e_TIMEOUT: "The scanner is not answering. Unplug and replug the USB cable, then try again.",
    e_NOT_FOUND: "Scanner not found. Check the USB cable and that the scanner is switched on.",
    e_LOCKED: "The scan could not start. Check that the scanner's transport LOCK switch is open and that the scanner is fully assembled (its own glass panel must be in place), then try again.",
    e_BUSY: "The scanner is busy. Wait a few seconds and try again.",
    e_DISK_FULL: "The disk is full. Free some space (one scan needs about 35 MB) or change the data folder in Settings.",
    e_SCAN_FAILED: "Scan failed", e_NOT_RESUMABLE: "This patient is already finished or cancelled.", e_BAD_STATION: "The laptop code needs 1–4 letters or digits.",
    e_BACKUP_SAME: "The backup folder must be different from the data folder (not inside it, not its parent).", e_BACKUP_FAILED: "The backup folder cannot be used", e_BACKUP_OFF: "No backup folder is set.",
    set: "Settings", setSub: "Settings: data folder, scanner, backup copy and scale calibration.",
    dataf: "Data folder", saved2: "Saved ✓", station: "Laptop code", stationNote: "Prefix of internal patient IDs, unique per laptop (1–4 letters/digits). Never the same as another laptop.",
    scanner: "Scanner", auto: "Automatic (LiDE 300)", refresh: "↻ Refresh",
    cal: "Scale calibration", calNote: "Calibrated on 13 Nov 2026 with a steel and a plastic ruler on the acrylic (final case), checked again on 18 Nov 2026 with the calibration sheet on the standing platform. Lengthwise (scanner travel): 300.7 dpi = 11.84 pixels per mm. Across: 299.5 dpi = 11.79 pixels per mm. These values are recorded in every patient file.",
    calCheck: "Check again after the scanner is moved, the spacer is changed or the acrylic is replaced: scan the calibration sheet or the steel ruler and tell the team if 1 mm is no longer 11.84 pixels.",
    savedBtn: "✓ Saved", browse: "Choose folder…", bkDone: "Warning: the second copy did not succeed. The data exists on one disk only until the backup folder is reachable again.",
    scChecking: "Checking scanner…", scReady: "Scanner ready", scMissing: "Scanner not detected", scAttn: "Scanner needs attention",
    scMissingMsg: "Unplug and replug the scanner's USB cable, and make sure the scanner is switched on.",
    scNullMsg: "Windows installed an empty driver (Null Driver), so the scanner cannot be used. Unplug and replug the USB cable; if it stays like this, call the team.",
    scPanel: "Scanner check", scState: "Status", scFound: "Devices detected", scNone: "none", scRoute: "Route in use", scNullRow: "Null Driver",
    scDriverless: "Driverless (preferred)", scCanon: "Canon driver", scOther: "Other", scNo: "none", scYes: "PRESENT, unplug and replug",
    scTest: "Test scanner", scTesting: "Testing… (up to ~25 s)", scTestNote: "Scans a small strip with no foot on the glass. Proves the scanner can really scan and the LOCK switch is open.",
    scLast: "Last test", scNever: "none yet", scPass: "passed", scFail: "failed",
    bkLabel: "Second data copy", bkNote: "A folder on another disk, USB drive or cloud folder. Every save is copied automatically. Leave empty to turn it off.",
    bkNow: "Back up now", bkNowBusy: "Copying…", bkOff: "No second copy set (the data lives on one disk only).", bkOk: "Last copy succeeded", bkFiles: "files", bkNever: "No copy yet since the app opened.",
    bkGone: "The backup folder cannot be reached (USB unplugged?). New data has not been copied.", bkFail: "The last copy failed",
    ddSaved: "Data saved to", ddChange: "Change", ddBad: "The data folder cannot be reached",
    ddBadHint: "Plug the flash disk or drive back in, or choose another folder with Change. Scanning cannot start yet.",
    setupT: "Welcome to SmartFoot", setupSub: "Before you start, choose where the scans are saved on this laptop.",
    setupData: "Data folder", required: "required", setupDataNote: "Every session and scan is saved here, on the laptop disk or on a flash disk.",
    setupBk: "Second copy (optional)", setupBkNote: "A folder on another disk, USB drive or cloud. Leave empty to skip; the data then lives on one disk only.",
    setupCode: "Laptop code", setupGo: "Save and start →", setupSaving: "Saving…",
    res: "Measurement details", resSub: "All numbers from the scans.", resSess: "Session", resPat: "Patient",
    adjust: "Adjust reference points", adjustOff: "Done adjusting",
    resNone: "No patient has been scanned yet.", resPts: "Drag the two white points: heel centre (bottom) and 2nd-toe tip (top), then release. Numbers are recalculated.",
    resCorr: "Points corrected", yes2: "yes", no2: "no", resAuto: "Reset to automatic",
    resUncal: "Not calibrated: numbers use nominal 300 dpi, do not use them as final results.", resNoMeas: "This scan could not be measured (no foot found).",
    resGap: "Toe base from the gaps between toes", resProp: "Toe base estimated at 82% of length (toes touching)",
  },
};

let L = localStorage.getItem("lang") || "id";
const t = (k) => T[L][k];
const $ = (s) => document.querySelector(s);
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

// ---- scanner status: chip in the top bar, gate on the patient form, panel in team settings ----
const sc = { phase: "checking", status: null, lock: false, test: null, testing: false };
const POLL_MS = 4000;
let bk = null; // second-copy status from Rust
let dd = null; // data folder: { path, ok }
const bkLine = () => !bk?.enabled ? t("bkOff") : !bk.reachable ? t("bkGone") : bk.last_err ? `${t("bkFail")}: ${esc(bk.last_err)}` : bk.last_ok ? `${t("bkOk")}: ${esc(bk.last_ok)} · ${bk.last_files} ${t("bkFiles")}` : t("bkNever");

// In Indonesian text the English word "scanner" is written in italics.
function italicize(root) {
  if (L !== "id" || !root) return;
  const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const hits = [];
  for (let n; (n = w.nextNode()); ) {
    if (/scanner/i.test(n.nodeValue) && !n.parentElement.closest("option,input,textarea,script,style,i.sc")) hits.push(n);
  }
  hits.forEach((n) => {
    const f = document.createDocumentFragment();
    n.nodeValue.split(/(scanner)/i).forEach((part, i) => {
      if (i % 2) { const e = document.createElement("i"); e.className = "sc"; e.textContent = part; f.append(e); }
      else if (part) f.append(document.createTextNode(part));
    });
    n.replaceWith(f);
  });
}

// "ready" | "attn" | "missing" | "checking"
function scLevel() {
  if (sc.phase === "checking") return "checking";
  const s = sc.status;
  if (!s || !s.chosen) return "missing";
  return sc.lock || s.null_driver || (sc.test && !sc.test.ok) ? "attn" : "ready";
}
function scMessage() {
  const lv = scLevel();
  if (lv === "missing") return sc.status?.null_driver ? t("scNullMsg") : t("scMissingMsg");
  if (lv === "attn") return sc.lock ? t("e_LOCKED") : sc.status?.null_driver ? t("scNullMsg") : `${t("scFail")}: ${errText(sc.test.err).replace(/<[^>]*>/g, "")}`;
  return "";
}
function paintChip() {
  const lv = scLevel(), b = $("#schip");
  if (!b) return;
  b.className = "schip " + lv;
  b.innerHTML = `<span class="d"></span>${esc(t({ ready: "scReady", attn: "scAttn", missing: "scMissing", checking: "scChecking" }[lv]))}`;
  b.title = scMessage();
  italicize(b);
}
function paintDD() {
  const el = $("#ddline"); if (!el) return;
  if (!dd) { el.innerHTML = ""; return; }
  el.innerHTML = dd.ok
    ? `<div class="ddline">📁 ${t("ddSaved")}: <b>${esc(dd.path)}</b> · <button class="linkbtn" id="ddchg" type="button">${t("ddChange")}</button></div>`
    : `<div class="err">📁 ${t("ddBad")}: <b>${esc(dd.path)}</b><br><span style="font-weight:500">${t("ddBadHint")}</span> <button class="linkbtn" id="ddchg" type="button">${t("ddChange")}</button></div>`;
  $("#ddchg").onclick = () => $("#teamBtn").click();
}
function paintGate() {
  paintDD();
  const g = $("#scgate"), go = $("#go"), lv = scLevel();
  if (g) {
    const msg = scMessage(), bad = bk?.enabled && (!bk.reachable || bk.last_err);
    g.innerHTML = (msg ? `<div class="${lv === "missing" ? "err" : "warn"}">${esc(msg)}</div>` : "") + (bad ? `<div class="warn">${bkLine()}</div>` : "");
    italicize(g);
  }
  if (go) go.toggleAttribute("disabled", st.busy || lv === "missing" || (dd && !dd.ok));
}
function paintPanel() {
  const el = $("#scpanel"); if (!el) return;
  const s = sc.status, lv = scLevel();
  const kv = (k, v) => `<div class="kv"><span>${t(k)}</span><b>${v}</b></div>`;
  const route = !s?.chosen ? "–" : esc(t({ driverless: "scDriverless", canon: "scCanon", other: "scOther" }[s.route] || "scOther"));
  const last = sc.test ? (sc.test.ok ? `${t("scPass")} · ${sc.test.seconds.toFixed(1)} s` : `${t("scFail")} · ${errText(sc.test.err)}`) : t("scNever");
  el.innerHTML = `<h2 style="font-size:20px;margin-bottom:10px">${t("scPanel")}</h2>
    ${kv("scState", `<span class="chip ${lv === "ready" ? "ok" : lv === "checking" ? "" : "warn"}">${esc(t({ ready: "scReady", attn: "scAttn", missing: "scMissing", checking: "scChecking" }[lv]))}</span>`)}
    ${kv("scFound", s?.devices?.length ? s.devices.map(esc).join("<br>") : t("scNone"))}
    ${kv("scRoute", route)}${kv("scNullRow", s?.null_driver ? `<span style="color:var(--bad)">${t("scYes")}</span>` : t("scNo"))}${kv("scLast", last)}
    <p class="muted" style="margin:12px 0 14px">${t("scTestNote")}</p>
    <button class="btn ghost" id="sctest" style="padding:12px 18px;font-size:16px" ${sc.testing || lv === "missing" ? "disabled" : ""}>${sc.testing ? t("scTesting") : t("scTest")}</button>`;
  $("#sctest").onclick = runScannerTest;
  italicize(el);
}
async function pollScanner() {
  if (st.busy || sc.testing || st.scanning) return; // never touch the scanner while it works
  try { sc.status = await invoke("scanner_status"); } catch { sc.status = null; }
  try { bk = await invoke("backup_status"); } catch { bk = null; }
  try { dd = await invoke("data_dir_status"); } catch { dd = null; }
  sc.phase = "done"; paintChip(); paintGate(); paintPanel();
}
async function runScannerTest() {
  sc.testing = true; paintPanel();
  try { const secs = await invoke("test_scanner"); sc.test = { ok: true, seconds: secs }; sc.lock = false; }
  catch (e) { sc.test = { ok: false, err: e }; if (String(e).startsWith("LOCKED")) sc.lock = true; }
  sc.testing = false; paintChip(); paintGate(); paintPanel();
}
async function refreshBk() { try { bk = await invoke("backup_status"); } catch { bk = null; } render(); }
function noteScanError(e) { if (String(e).startsWith("LOCKED")) { sc.lock = true; paintChip(); } }

// ---- app state ----
// sesi = SesiState from Rust (open session), pat = PatientInfo (patient being worked on),
// side = foot on the scan/review screens, scan = ScanOutcome waiting for Accept/Retake.
const st = {
  screen: "boot", prev: "home", sesi: null, pat: null, side: null, scan: null, error: "", busy: false, scanning: false,
  settings: null, draft: {}, flash: null, ferr: null, scanners: [], damaged: [], warn: [], spec: null,
  home: { place: "", op: "" }, notice: null, form: null, editBack: null, inl: null, inlDirty: false, nt: null, ntDirty: false, views: {},
};
// text fields in Settings with their own Save button: current value and "unchanged" test
const FIELDS = {
  dir: { get: (c) => c.data_dir, same: (v, c) => v.trim() === c.data_dir },
  station: { get: (c) => c.station, same: (v, c) => v.trim().toUpperCase() === c.station },
  bk: { get: (c) => c.backup_dir ?? "", same: (v, c) => v.trim() === (c.backup_dir ?? "") },
};

function errText(raw) {
  if (/not enough space|disk is full|os error 112/i.test(String(raw))) return esc(t("e_DISK_FULL"));
  const m = String(raw).match(/^([A-Z_]+):\s*(.*)$/s);
  if (!m) return esc(raw);
  const known = T[L]["e_" + m[1]];
  const withDetail = ["SCAN_FAILED", "BACKUP_FAILED", "PICTURE", "FILE_OPEN"].includes(m[1]);
  return esc(known ? (withDetail ? `${known}: ${m[2]}` : known) : raw);
}
const errBox = () => (st.error ? `<div class="err">${errText(st.error)}</div>` : "");
// file warnings from the last save (Excel open, picture failed): shown once, then cleared
const warnBox = () => { const w = st.warn; st.warn = []; return w.length ? `<div class="warn">${w.map(errText).join("<br>")}</div>` : ""; };
const takeWarn = (r) => { if (r?.file_warnings?.length) st.warn = r.file_warnings; return r; };

// ---- small helpers ----
const sideName = (s) => t(s === "left" ? "left" : "right");
const hasScan = (p, side) => !!p?.scans?.some((e) => e.side === side);
const isPaused = (s) => { const p = s?.sesi?.pauses; return !!p?.length && !p[p.length - 1].end; };
const fmtDate = (iso) => { const d = new Date(iso.length <= 10 ? iso + "T00:00:00" : iso); return d.toLocaleDateString(L === "id" ? "id-ID" : "en-GB", { weekday: "long", day: "numeric", month: "long", year: "numeric" }); };
const fmtTime = (iso) => (iso ? iso.slice(11, 16).replace(":", L === "id" ? "." : ":") : "");
const sbtn = "padding:12px 20px;font-size:16px";
function armed(btn, label, sure, act) {
  if (!btn) return; let on = false;
  btn.onclick = async () => {
    if (!on) { on = true; btn.textContent = sure; btn.classList.add("armed"); setTimeout(() => { on = false; if (btn.isConnected) { btn.textContent = label; btn.classList.remove("armed"); } }, 4000); return; }
    btn.disabled = true; await act();
  };
}

// INLOW, the same scoring as src-tauri/src/inlow.rs (Rust stays the source of truth for the Excel)
const RISK = { id: [[6, "A", "Risiko sangat rendah"], [12, "B", "Risiko rendah"], [19, "C", "Risiko sedang"], [25, "D", "Risiko tinggi"], [1e9, "E", "Risiko urgen"]],
  en: [[6, "A", "Very low risk"], [12, "B", "Low risk"], [19, "C", "Moderate risk"], [25, "D", "High risk"], [1e9, "E", "Urgent risk"]] };
const footFilled = (f) => st.spec.items.filter((i) => f.items[i.key] != null).length;
const footTotal = (f) => (footFilled(f) === st.spec.items.length ? Object.values(f.items).reduce((a, b) => a + b, 0) : null);
const riskOf = (tot) => RISK[L].find((r) => tot <= r[0]);
const inlowComplete = (inl) => !!st.spec && footTotal(inl.kanan) != null && footTotal(inl.kiri) != null;
const blankFoot = () => ({ items: {}, questions: [null, null, null, null] });
const copyInlow = (i) => ({ kanan: { items: { ...(i?.kanan?.items ?? {}) }, questions: [...(i?.kanan?.questions ?? [null, null, null, null])] }, kiri: { items: { ...(i?.kiri?.items ?? {}) }, questions: [...(i?.kiri?.questions ?? [null, null, null, null])] } });

// ---- step bar: the patient's six steps; clickable once the patient exists ----
const PATIENT_SCREENS = ["pform", "scan", "turn", "review", "check", "inlow", "notes", "finished"];
function stepIndex() {
  switch (st.screen) {
    case "pform": return 0;
    case "scan": case "review": case "check": return st.side === "left" ? 1 : 2;
    case "turn": return 2;
    case "inlow": return 3;
    case "notes": return 4;
    case "finished": return 5;
    default: return -1;
  }
}
function stepDone(i) {
  const p = st.pat?.patient; if (!p) return false;
  return [true, hasScan(p, "left"), hasScan(p, "right"), inlowComplete(p.inlow), !!(p.notes?.kiri || p.notes?.kanan || p.notes?.umum || p.marks?.length), p.completed][i];
}
function stepReachable(i) {
  const p = st.pat?.patient;
  if (!p || !PATIENT_SCREENS.includes(st.screen) || st.screen === "review" || st.screen === "check" || st.scanning || st.busy || i === stepIndex()) return false;
  if (i === 1 || i === 2) return st.pat.next_side === (i === 1 ? "left" : "right");
  if (i === 5) return !st.pat.next_side;
  return true;
}
function steps() {
  const p = stepIndex();
  $("#steps").classList.toggle("off", !st.pat || !PATIENT_SCREENS.includes(st.screen) || (st.screen === "check" && !!ck.back));
  $("#steps").innerHTML = t("steps").map((n, i) => {
    const done = stepDone(i) && i !== p, go = stepReachable(i);
    return `<button type="button" class="step ${done ? "done" : i == p ? "on" : ""} ${go ? "go" : ""}" data-step="${i}" ${go ? "" : "tabindex='-1'"}><i>${done ? "✓" : i + 1}</i><b>${n}</b></button>`;
  }).join("");
  $("#steps").querySelectorAll(".step.go").forEach((b) => (b.onclick = () => goStep(Number(b.dataset.step))));
}
async function goStep(i) {
  if (!(await saveDrafts())) return;
  st.error = "";
  if (i === 0) { openForm(true); return; }
  if (i === 1 || i === 2) { st.side = i === 1 ? "left" : "right"; st.scan = null; st.screen = "scan"; }
  if (i === 3) openInlow();
  if (i === 4) { openNotes(); return; }
  if (i === 5) { await finishPatient(); return; }
  render();
}

// ---- loading the session ----
async function loadSesi() {
  try { st.sesi = await invoke("current_sesi"); } catch (e) { st.error = e; st.sesi = null; }
  try { st.damaged = await invoke("damaged_patients"); } catch { st.damaged = []; }
}
function hubScreen() { return !st.sesi ? "home" : st.sesi.stale ? "stale" : isPaused(st.sesi) ? "paused" : "sesi"; }
async function toHub() { st.pat = null; st.scan = null; st.side = null; await loadSesi(); st.screen = hubScreen(); render(); }

// ---- patient actions ----
function openForm(edit) {
  const p = edit ? st.pat?.patient : null;
  st.form = p ? { nama: p.nama, usia: String(p.usia), kelamin: p.kelamin, td: p.td, bb: p.bb == null ? "" : String(p.bb), posisi: p.posisi, edit: true }
    : { nama: "", usia: "", kelamin: "", td: "", bb: "", posisi: localStorage.getItem("lastPosisi") || "berdiri", edit: false };
  st.editBack = edit ? st.screen : null;
  if (!edit) st.pat = null;
  st.error = ""; st.screen = "pform"; render();
}
function openInlow() { st.inl = copyInlow(st.pat.patient.inlow); st.inlDirty = false; st.screen = "inlow"; }
function openNotes() {
  const p = st.pat.patient;
  st.nt = { notes: { kiri: p.notes?.kiri ?? "", kanan: p.notes?.kanan ?? "", umum: p.notes?.umum ?? "" }, marks: (p.marks ?? []).map((m) => ({ ...m })), marking: false, label: "kalus", other: "" };
  st.ntDirty = false; st.screen = "notes"; render(); loadViews();
}
// keep the INLOW / notes drafts before leaving those screens; false = the save failed (stay)
async function saveDrafts() {
  try {
    if (st.screen === "inlow" && st.inlDirty) { st.pat = takeWarn(await invoke("save_inlow", { key: st.pat.key, inlow: st.inl })); st.inlDirty = false; }
    if (st.screen === "notes" && st.ntDirty) {
      keepNotes();
      await invoke("save_notes", { key: st.pat.key, notes: st.nt.notes });
      st.pat = takeWarn(await invoke("save_marks", { key: st.pat.key, marks: st.nt.marks }));
      st.ntDirty = false;
    }
    return true;
  } catch (e) { st.error = e; render(); return false; }
}
function keepNotes() { ["kiri", "kanan", "umum"].forEach((k) => { const el = $("#n_" + k); if (el) st.nt.notes[k] = el.value; }); }
async function finishPatient() {
  try {
    st.pat = takeWarn(await invoke("finish_patient", { key: st.pat.key }));
    await loadSesi(); st.error = ""; st.screen = "finished";
    try { bk = await invoke("backup_status"); } catch { bk = null; }
  } catch (e) { st.error = e; }
  render();
}
async function cancelPatient() {
  try { await invoke("cancel_patient", { key: st.pat.key }); } catch (e) { st.error = e; render(); return; }
  st.error = ""; await toHub();
}
async function resumePatient(key) {
  try {
    const r = await invoke("resume_patient", { key });
    st.pat = r.info; st.side = r.info.next_side; st.error = "";
    if (r.pending) { st.scan = r.pending; st.screen = "review"; }
    else if (st.side === "right" && hasScan(r.info.patient, "left")) { st.screen = "turn"; }
    else if (st.side) { st.scan = null; st.screen = "scan"; }
    else { openInlow(); }
  } catch (e) { st.error = e; }
  render();
}
async function openPatient(key) {
  try { st.pat = await invoke("get_patient", { key }); st.error = ""; openInlow(); } catch (e) { st.error = e; }
  render();
}
// scan pictures for the marking screen (1/4 size), cached per patient + foot
async function loadViews() {
  for (const side of ["left", "right"]) {
    const id = st.pat.key + "|" + side;
    if (!hasScan(st.pat.patient, side) || st.views[id]) continue;
    try { st.views[id] = await invoke("get_scan_view", { key: st.pat.key, side }); } catch { continue; }
    if (st.screen === "notes") paintMarks();
  }
}

// ---- outline of a foot as the placing guide on the scan screen ----
const OUT = "M112 258c-16 8-38 6-44-12-6-20 4-44 2-70-2-26-20-52-12-86 7-30 34-44 56-36 24 8 30 40 26 70-3 24 6 46 8 70 2 22-8 56-36 64z";
const guideFoot = (side) => `<svg viewBox="0 0 210 297"><g transform="${side == "left" ? "translate(210,0) scale(-1,1)" : ""}" opacity=".55"><path d="${OUT}" fill="#d7b79e"/><g fill="#d7b79e"><ellipse cx="114" cy="62" rx="13" ry="16"/><ellipse cx="135" cy="70" rx="8" ry="11"/><ellipse cx="148" cy="82" rx="7" ry="10"/><ellipse cx="156" cy="97" rx="6" ry="9"/><ellipse cx="160" cy="112" rx="5" ry="8"/></g></g></svg>`;
const turnPic = `<svg viewBox="0 0 240 262"><rect x="40" y="30" width="160" height="180" rx="14" fill="#e8f0fb" stroke="#1f5fbf" stroke-width="3"/><rect x="120" y="30" width="80" height="180" rx="0" fill="#cfe0f7"/>
  <g transform="translate(78,148) scale(.32) rotate(180)"><path d="${OUT}" transform="translate(-112,-150)" fill="#d7a98a"/></g>
  <g transform="translate(160,92) scale(.32)"><path d="${OUT}" transform="translate(-112,-150)" fill="#d7a98a"/></g>
  <path d="M70 222 Q120 262 172 224" stroke="#ffb703" stroke-width="7" fill="none" marker-end="url(#tarr)"/>
  <defs><marker id="tarr" viewBox="0 0 10 10" refX="6" refY="5" markerWidth="3.2" markerHeight="3.2" orient="auto"><path d="M0,0L10,5L0,10z" fill="#ffb703"/></marker></defs></svg>`;

// ---- measurement details ("Detail pengukuran"): all values, the point check, "Belum diperiksa" filter ----
const rs = { list: [], sesi: null, key: null, pat: null, side: null, view: null, only: false, back: "sesi" };
const rsSessions = () => rs.list.filter((s) => s.patients.some((p) => p.scans > 0 && (!rs.only || p.unchecked.length)));
const rsPatients = () => (rs.list.find((s) => s.key === rs.sesi)?.patients ?? []).filter((p) => p.scans > 0 && (!rs.only || p.unchecked.length));
async function loadView() { rs.view = rs.key && rs.side ? await invoke("get_scan_view", { key: rs.key, side: rs.side }) : null; }
async function loadPatientRs() {
  rs.pat = rs.key ? await invoke("get_patient", { key: rs.key }) : null;
  const sides = rs.pat?.patient.scans.map((e) => e.side) ?? [];
  const row = rsPatients().find((p) => p.key === rs.key);
  if (rs.only && row && !row.unchecked.includes(rs.side)) rs.side = row.unchecked[0];
  if (!sides.includes(rs.side)) rs.side = sides.includes("left") ? "left" : sides[0] ?? null;
  await loadView();
}
// keep the current session / patient when they still pass the filter, else take the first that does
function rsPick(key) {
  const S = rsSessions(), owner = key && S.find((s) => s.patients.some((p) => p.key === key));
  rs.sesi = owner?.key ?? (S.find((s) => s.key === rs.sesi) ? rs.sesi : S[0]?.key ?? null);
  const P = rsPatients();
  rs.key = key && owner ? key : P.find((p) => p.key === rs.key)?.key ?? P[0]?.key ?? null;
}
async function openResearch(key, back, only) {
  if (back !== undefined) rs.back = back;
  if (only !== undefined) rs.only = only;
  try { rs.list = await invoke("list_all"); rsPick(key); await loadPatientRs(); st.error = ""; } catch (e) { st.error = e; }
  st.screen = "research"; render();
}
function overlayAndNumbers() {
  const v = rs.view, m = v?.measurements;
  if (!v) return { svg: "", nums: "" };
  let g = "";
  if (m) {
    const d = v.down, pt = (q) => [q[0] / d, q[1] / d], H = pt(m.heel_centre), J = pt(m.toe_tip), E = pt(m.heel_end);
    const l = Math.hypot(J[0] - H[0], J[1] - H[1]) || 1, a = [(J[0] - H[0]) / l, (J[1] - H[1]) / l], b = [-a[1], a[0]];
    const ppm = 1 / (m.scale.x_mm_per_px * d), half = (m.fw_mm / 2) * 1.15 * ppm, Lt = m.toeless_frac * m.fl_mm * ppm;
    for (let i = 0; i <= 3; i++) {
      const P = [E[0] + a[0] * Lt * i / 3, E[1] + a[1] * Lt * i / 3];
      g += `<line x1="${P[0] - b[0] * half}" y1="${P[1] - b[1] * half}" x2="${P[0] + b[0] * half}" y2="${P[1] + b[1] * half}" stroke="${i == 3 ? "#ffd166" : "#ffffff"}" stroke-width="${i == 3 ? 3 : 1.5}" stroke-dasharray="${i == 3 ? "8 5" : "3 5"}" opacity=".85"/>`;
    }
    g += `<line x1="${E[0]}" y1="${E[1]}" x2="${J[0]}" y2="${J[1]}" stroke="#fff" stroke-width="2" stroke-dasharray="6 4" opacity=".8"/>`;
    const c = v.check?.status === "diperiksa" ? v.check.clarke_pts : null;
    if (c) { const [F, Hh, A] = [pt(c.fore), pt(c.heel), pt(c.arch)];
      g += `<line x1="${F[0]}" y1="${F[1]}" x2="${Hh[0]}" y2="${Hh[1]}" stroke="#ffb703" stroke-width="3"/><line x1="${F[0]}" y1="${F[1]}" x2="${A[0]}" y2="${A[1]}" stroke="#ffb703" stroke-width="3"/>${[F, Hh, A].map((p) => `<circle cx="${p[0]}" cy="${p[1]}" r="7" fill="#ffb703"/>`).join("")}`; }
  }
  const svg = `<svg viewBox="0 0 ${v.preview_w} ${v.preview_h}" style="width:100%;height:100%"><image href="${v.preview}" width="${v.preview_w}" height="${v.preview_h}"/>${g}</svg>`;
  const kv = (k) => { const [val, d, u] = VAL[k](m); return `<div class="kv"><span style="display:flex;align-items:center">${esc(X[L]["i_" + k][0])}${ib(k)}</span><b>${num(val, d)}${u ? " " + u : ""} <em class="mp2">${precText(k)}</em></b></div>`; };
  const nums = !m ? `<div class="warn">${t("resNoMeas")}</div>` :
    `${["fl", "fw", "area", "toeless", "A", "B", "C", "ai", "csi"].map(kv).join("")}
     <p class="muted" style="margin:12px 0 4px">${m.toe_base_method === "gap" ? t("resGap") : t("resProp")}</p>
     <span class="chip ${m.corrected ? "warn" : "ok"}">${t("resCorr")}: ${m.corrected ? t("yes2") : t("no2")}</span>
     ${m.scale.calibrated ? "" : `<div class="warn" style="margin-top:14px">${t("resUncal")}</div>`}`;
  return { svg, nums };
}
// the latest point check of the shown foot, and how many earlier checks it replaced
function checkCard() {
  const v = rs.view; if (!v?.measurements) return "";
  const e = rs.pat?.patient.scans.find((x) => x.side === rs.side), c = v.check, hist = e?.check_history?.length ?? 0;
  const when = c ? `${esc(c.operator)} · ${esc(c.at.slice(0, 10))} ${fmtTime(c.at)}` : "";
  const state = !c ? `<span class="chip warn">${t("stNone")}</span>`
    : c.status === "diperiksa" ? `<span class="chip ok">${t("okChk")}</span> ${t("clarke")} <b>${degText(c.clarke_deg)}</b> (${t("lbl")[clarkeLabel(c.clarke_deg)]}) · ${when}`
    : `<span class="chip warn">${t("okSkip")}</span> ${esc(t("skipR")[c.skip_reason] ?? c.skip_reason ?? "")} · ${when}`;
  return `<div class="card ckcard"><div class="ml" style="margin-bottom:8px">${t("ptCheck")}</div><div class="row" style="gap:8px">${state}</div>
    ${hist ? `<p class="muted" style="margin:8px 0 0">${hist} ${t("hist")}</p>` : ""}
    <div class="row" style="margin-top:14px"><button class="btn ${c?.status === "diperiksa" ? "ghost" : ""}" id="rs_chk" style="${sbtn}">${c?.status === "diperiksa" ? t("recheck") : t("chk")} →</button></div></div>`;
}
function researchScreen(m) {
  const back = () => { st.screen = rs.back; render(); };
  const filter = `<label class="ly" style="flex:none"><input type="checkbox" id="rs_only" ${rs.only ? "checked" : ""}> ${t("onlyUnchk")}</label>`;
  if (!rsSessions().length) {
    m.innerHTML = `<h1>${t("res")}</h1>${errBox()}<div class="row" style="margin-bottom:16px">${filter}</div><div class="card"><p class="sub" style="margin:0">${rs.only ? t("allChecked") : t("resNone")}</p></div><div class="row" style="margin-top:22px"><button class="btn ghost" id="rback" style="${sbtn}">${t("back")}</button></div>`;
    $("#rback").onclick = back; $("#rs_only").onchange = (e) => openResearch(null, undefined, e.target.checked); return;
  }
  const scans = rs.pat?.patient.scans ?? [], row = rsPatients().find((p) => p.key === rs.key);
  const sideBtn = (s) => `<button data-sd="${s}" class="${rs.side == s ? "on" : ""}" style="padding:12px 16px;font-size:16px;white-space:nowrap">${sideName(s)}${row?.unchecked.includes(s) ? ' <span class="udot"></span>' : ""}</button>`;
  m.innerHTML = `<h1>${t("res")}</h1><p class="sub">${t("resSub")}</p>${noticeBox()}${errBox()}
    <div class="row" style="margin-bottom:16px">${filter}
      <label style="margin:0">${t("resSess")}</label><select id="rs_sess" style="width:auto">${rsSessions().map((s) => `<option value="${esc(s.key)}" ${s.key == rs.sesi ? "selected" : ""}>${esc(s.date)} · ${esc(s.place)}</option>`).join("")}</select>
      <label style="margin:0">${t("resPat")}</label><select id="rs_pat" style="width:auto">${rsPatients().map((p) => `<option value="${esc(p.key)}" ${p.key == rs.key ? "selected" : ""}>${String(p.no).padStart(3, "0")} ${esc(p.nama)}${p.unchecked.length ? " ●" : ""}</option>`).join("")}</select>
      <div class="seg" style="flex:none">${["left", "right"].filter((s) => scans.some((e) => e.side === s)).map(sideBtn).join("")}</div></div>
    <div class="grid2"><div class="card"><div class="scanwin" id="rsimg" style="background:#10151f"></div></div>
    <div>${checkCard()}<div class="card" id="rsnums" style="margin-top:18px"></div></div></div>
    <div class="row" style="margin-top:22px"><button class="btn ghost" id="rback" style="${sbtn}">${t("back")}</button></div>`;
  const o = overlayAndNumbers();
  $("#rsimg").innerHTML = o.svg; $("#rsnums").innerHTML = o.nums;
  const v = rs.view;
  if (v?.measurements) wireInfo($("#rsnums"), { img: v.preview, w: v.preview_w, h: v.preview_h, down: v.down, m: v.measurements });
  const reload = async (f) => { try { await f(); st.error = ""; } catch (x) { st.error = x; } render(); };
  $("#rs_only").onchange = (e) => reload(async () => { rs.only = e.target.checked; rsPick(rs.key); await loadPatientRs(); });
  $("#rs_sess").onchange = (e) => reload(async () => { rs.sesi = e.target.value; rs.key = rsPatients()[0]?.key ?? null; await loadPatientRs(); });
  $("#rs_pat").onchange = (e) => reload(async () => { rs.key = e.target.value; await loadPatientRs(); });
  m.querySelectorAll("[data-sd]").forEach((b) => (b.onclick = () => reload(async () => { rs.side = b.dataset.sd; await loadView(); })));
  if ($("#rs_chk")) $("#rs_chk").onclick = () => checkLater(rs.pat, rs.side, () => openResearch(rs.key));
  $("#rback").onclick = back;
}
// open the point check outside the patient flow (from Detail pengukuran or the Akhiri sesi reminder);
// `then` runs after confirm, skip or Kembali, with the patient flow state put back
function checkLater(pat, side, then) {
  const keep = { pat: st.pat, side: st.side };
  st.pat = pat;
  openCheck(side, null, () => { st.pat = keep.pat; st.side = keep.side; then(); });
}

// ---- Akhiri sesi: remind about feet nobody has checked ----
async function endSession(after) {
  let list = [];
  try { list = await invoke("unchecked_feet"); } catch (e) { st.error = e; render(); return; }
  const end = async () => {
    try { st.warn = await invoke("end_sesi"); st.error = ""; closeModal(); await loadSesi(); st.screen = after ?? "ended"; } catch (e) { closeModal(); st.error = e; }
    render();
  };
  if (!list.length) return end();
  modal(`<div class="mhead"><h3>${t("remT")}</h3><button class="btn ghost mx" id="mx">✕</button></div><p style="line-height:1.5;margin-bottom:14px">${t("remSub")}</p>
    <div class="remlist">${list.map((f, i) => `<div class="remrow"><div><b>${String(f.no).padStart(3, "0")} ${esc(f.nama)}</b> · ${sideName(f.side)}<div class="muted">${f.status === "dilewati" ? `${t("okSkip")}: ${esc(t("skipR")[f.skip_reason] ?? f.skip_reason ?? "")}` : t("stNone")}</div></div>
      <button class="btn ghost right" data-rem="${i}" style="padding:9px 16px;font-size:15px">${t("chk")} →</button></div>`).join("")}</div>
    <div class="row" style="margin-top:20px"><button class="btn ghost" id="rem_back" style="${sbtn}">${t("back")}</button><button class="btn bad right" id="rem_end" style="${sbtn}">${t("remEnd")}</button></div>`, "wide");
  $("#mx").onclick = $("#rem_back").onclick = () => { closeModal(); render(); };
  $("#rem_end").onclick = () => { $("#rem_end").disabled = true; end(); };
  document.querySelectorAll("#modal [data-rem]").forEach((b) => (b.onclick = async () => {
    const f = list[Number(b.dataset.rem)];
    try { const pat = await invoke("get_patient", { key: f.key }); closeModal(); checkLater(pat, f.side, async () => { await toHub(); endSession(after); }); }
    catch (e) { closeModal(); st.error = e; render(); }
  }));
}

// ---- screens ----
function homeScreen(m) {
  const h = st.home, ops = st.settings?.operators ?? [], now = new Date().toISOString();
  const dmg = st.damaged?.length ? `<div class="warn">${t("dmg")}: <b>${st.damaged.map(esc).join(", ")}</b>. ${t("dmgHint")}</div>` : "";
  m.innerHTML = `<h1>${t("home")}</h1><p class="sub">${t("homeSub")}</p><div id="ddline"></div>${dmg}${warnBox()}${errBox()}<div id="herr"></div>
    <div class="card" style="max-width:820px">
      <label>${t("when")} <span class="muted">(${t("whenNote")})</span></label><div class="readonly">${esc(fmtDate(now.slice(0, 10)))} · ${new Date().toLocaleTimeString(L === "id" ? "id-ID" : "en-GB", { hour: "2-digit", minute: "2-digit" })}</div>
      <div class="field" style="margin-top:18px"><label>${t("place")}</label><input id="h_place" value="${esc(h.place)}" placeholder="${t("placePh")}" maxlength="60"></div>
      <div class="field"><label>${t("op")}</label><input id="h_op" value="${esc(h.op)}" placeholder="${t("opPh")}" maxlength="60">
        ${ops.length ? `<div class="muted" style="margin:10px 0 6px">${t("opPick")}</div><div class="chips">${ops.map((o) => `<button type="button" class="pick ${o === h.op ? "on" : ""}" data-op="${esc(o)}">${esc(o)}</button>`).join("")}</div>` : ""}</div>
      <div class="row"><button class="btn right" id="go_sesi">${t("goSesi")}</button></div></div>`;
  const keep = () => { h.place = $("#h_place").value; h.op = $("#h_op").value; };
  m.querySelectorAll("[data-op]").forEach((b) => (b.onclick = () => { keep(); h.op = b.dataset.op; render(); }));
  paintDD();
  $("#go_sesi").onclick = async () => {
    keep();
    const bad = !h.place.trim() ? "e_PLACE" : !h.op.trim() ? "e_OPNAME" : null;
    if (bad) { $("#herr").innerHTML = `<div class="err">${t(bad)}</div>`; return; }
    const b = $("#go_sesi"); b.disabled = true; b.textContent = t("startingSesi");
    try {
      st.sesi = takeWarn(await invoke("start_sesi", { place: h.place, operator: h.op }));
      st.settings = await invoke("get_settings"); st.error = ""; st.screen = "sesi"; h.place = "";
    } catch (e) { st.error = e; }
    render();
  };
}

function unfinishedBanner(key) {
  const row = st.sesi.patients.find((p) => p.key === key);
  return `<div class="card" style="margin-bottom:20px;border:2px solid var(--warn)"><div class="row"><div><div style="font-size:20px;font-weight:800">${t("unf")}</div>
    <div class="muted" style="margin:4px 0 8px">${t("unfSub")}</div><div><b>${String(row?.no ?? "").padStart(3, "0")} ${esc(row?.nama ?? key)}</b> · ${row?.scans ?? 0}/2 · ${t("hubOp")} ${esc(row?.operator ?? "")}</div></div>
    <div class="row right"><button class="btn bad" id="unf_cancel" style="${sbtn}">${t("cancelP")}</button><button class="btn" id="unf_go" style="${sbtn}">${t("resume")} →</button></div></div></div>`;
}
function wireUnfinished(key) {
  if (!$("#unf_go")) return;
  $("#unf_go").onclick = () => { $("#unf_go").disabled = true; $("#unf_go").textContent = t("resuming"); resumePatient(key); };
  armed($("#unf_cancel"), t("cancelP"), t("cancelSure"), async () => {
    try { await invoke("cancel_patient", { key }); st.error = ""; } catch (e) { st.error = e; }
    await toHub();
  });
}
function patientTable() {
  const P = st.sesi.patients;
  if (!P.length) return `<p class="muted">${t("none")}</p>`;
  return `<table class="plist"><thead><tr><th>${t("cNo")}</th><th>${t("cName")}</th><th>${t("cOp")}</th><th>${t("cScans")}</th><th>${t("cInlow")}</th><th>${t("cConc")}</th><th></th></tr></thead><tbody>
    ${P.map((p) => `<tr class="${p.cancelled ? "gone" : ""}"><td>${String(p.no).padStart(3, "0")}</td><td><b>${esc(p.nama)}</b>${p.cancelled ? ` <span class="muted">(${t("cancelled")})</span>` : !p.completed ? ` <span class="chip warn">${t("running")}</span>` : ""}</td>
      <td>${esc(p.operator)}</td><td>${p.scans}/2</td><td>${p.cancelled ? "–" : p.inlow_complete ? `<span class="chip ok">${t("inlowOk")}</span>` : `<span class="chip warn">${t("inlowNo")}</span>`}</td>
      <td class="conc">${p.cancelled ? "–" : esc(p.kesimpulan)}</td><td>${p.cancelled || !p.completed ? "" : `<button class="btn ghost" data-open="${esc(p.key)}" style="padding:8px 14px;font-size:14px">${t("open")}</button>`}</td></tr>`).join("")}
    </tbody></table>`;
}
function sesiScreen(m) {
  const S = st.sesi, s = S.sesi;
  m.innerHTML = `<div class="row" style="align-items:flex-end"><div><h1>${esc(s.place)}</h1><p class="sub" style="margin-bottom:0">${esc(fmtDate(s.date))} · ${t("since")} ${fmtTime(s.started)}</p></div>
      <div class="row right"><button class="btn ghost" id="h_pause" style="${sbtn}">⏸ ${t("pause")}</button><button class="btn bad" id="h_end" style="${sbtn}">${t("endS")}</button></div></div>
    <div style="height:18px"></div>${warnBox()}${errBox()}
    ${S.unfinished ? unfinishedBanner(S.unfinished) : ""}
    <div class="card" style="margin-bottom:18px"><div class="row"><div><div class="muted">${t("hubOp")}</div><div style="font-size:24px;font-weight:800">${esc(s.operator)}</div></div>
      <button class="btn ghost" id="h_op" style="${sbtn}">${t("chOp")}</button><button class="btn ghost" id="h_det" style="${sbtn}">${t("resBtn")}</button>
      <button class="btn right" id="h_new" ${S.unfinished ? "disabled" : ""}>${t("newP")}</button></div></div>
    <div class="card"><h2 style="font-size:20px;margin-bottom:12px">${t("list")}</h2>${patientTable()}</div>`;
  wireUnfinished(S.unfinished);
  $("#h_new").onclick = () => openForm(false);
  m.querySelectorAll("[data-open]").forEach((b) => (b.onclick = () => openPatient(b.dataset.open)));
  $("#h_pause").onclick = async () => { try { st.sesi = takeWarn(await invoke("pause_sesi", { pause: true })); st.error = ""; st.screen = "paused"; } catch (e) { st.error = e; } render(); };
  $("#h_op").onclick = () => changeOperatorModal();
  $("#h_det").onclick = () => openResearch(null, "sesi");
  armed($("#h_end"), t("endS"), t("endSure"), () => endSession("ended"));
}
function changeOperatorModal() {
  const ops = st.settings?.operators ?? [];
  modal(`<div class="mhead"><h3>${t("chOpT")}</h3><button class="btn ghost mx" id="mx">✕</button></div><p class="muted" style="margin-bottom:14px">${t("chOpSub")}</p>
    <div id="operr"></div><input id="op_new" placeholder="${t("opPh")}" maxlength="60">
    ${ops.length ? `<div class="muted" style="margin:12px 0 6px">${t("opPick")}</div><div class="chips">${ops.map((o) => `<button type="button" class="pick" data-op="${esc(o)}">${esc(o)}</button>`).join("")}</div>` : ""}
    <div class="row" style="margin-top:20px"><button class="btn right" id="op_ok" style="${sbtn}">${t("ok")}</button></div>`);
  $("#mx").onclick = closeModal;
  document.querySelectorAll("#modal [data-op]").forEach((b) => (b.onclick = () => { $("#op_new").value = b.dataset.op; }));
  $("#op_new").focus();
  $("#op_ok").onclick = async () => {
    const v = $("#op_new").value;
    if (!v.trim()) { $("#operr").innerHTML = `<div class="err">${t("e_OPNAME")}</div>`; return; }
    try { st.sesi = takeWarn(await invoke("change_operator", { operator: v })); st.settings = await invoke("get_settings"); st.error = ""; closeModal(); render(); }
    catch (e) { $("#operr").innerHTML = `<div class="err">${errText(e)}</div>`; }
  };
}

function staleScreen(m) {
  const S = st.sesi, s = S.sesi;
  m.innerHTML = `<h1>${t("stale")}</h1><p class="sub">${t("staleSub")}</p>${errBox()}
    ${S.unfinished ? unfinishedBanner(S.unfinished) : ""}
    <div class="card"><div class="row"><div><div style="font-size:22px;font-weight:800">${esc(s.place)}</div><div class="muted">${esc(fmtDate(s.date))} · ${S.patients.filter((p) => !p.cancelled).length} ${t("patients")} · ${t("hubOp")} ${esc(s.operator)}</div></div>
      <button class="btn right" id="st_end" ${S.unfinished ? "disabled" : ""}>${t("endOld")}</button></div></div>`;
  wireUnfinished(S.unfinished);
  $("#st_end").onclick = () => endSession("home");
}

function pausedScreen(m) {
  const p = st.sesi.sesi.pauses.at(-1);
  m.innerHTML = `<div class="lock"><div class="big">⏸</div><h1>${t("paused")}</h1><p class="sub">${t("pausedSub")}</p>
    <p style="font-size:18px;margin-bottom:26px">${esc(st.sesi.sesi.place)} · ${t("since")} <b>${fmtTime(p?.start)}</b></p>${errBox()}
    <button class="btn" id="unpause">${t("unpause")}</button></div>`;
  $("#unpause").onclick = async () => { try { st.sesi = takeWarn(await invoke("pause_sesi", { pause: false })); st.error = ""; st.screen = "sesi"; } catch (e) { st.error = e; } render(); };
}

function endedScreen(m) {
  m.innerHTML = `<h1>${t("ended")}</h1><p class="sub">${t("endedSub")}</p>${warnBox()}${errBox()}<div id="ddline"></div>
    <div class="row" style="margin-top:12px"><button class="btn ghost" id="e_det">${t("resBtn")}</button><button class="btn right" id="e_new">${t("home")} →</button></div>`;
  paintDD();
  $("#e_det").onclick = () => openResearch(null, "ended");
  $("#e_new").onclick = () => { st.screen = "home"; render(); };
}

function formScreen(m) {
  const f = st.form, edit = f.edit;
  const seg = (k, opts) => `<div class="seg">${opts.map(([v, lab]) => `<button type="button" class="${f[k] === v ? "on" : ""}" data-${k}="${v}">${lab}</button>`).join("")}</div>`;
  m.innerHTML = `<h1>${edit ? t("pEdit") : t("pform")}</h1><p class="sub">${t("pformSub")}</p>${edit ? "" : `<div id="ddline"></div><div id="scgate"></div>`}${errBox()}<div id="ferr"></div>
    <div class="card"><div class="grid2">
      <div class="field" style="grid-column:1/-1"><label>${t("nama")} <span class="muted">(${t("req")})</span></label><input id="f_nama" value="${esc(f.nama)}" maxlength="60" autocomplete="off"></div>
      <div class="field"><label>${t("age")} <span class="muted">(${t("req")})</span></label><div class="unit"><input id="f_usia" type="number" min="0" max="120" value="${esc(f.usia)}" placeholder="45"><span>${t("ageU")}</span></div></div>
      <div class="field"><label>${t("sex")} <span class="muted">(${t("req")})</span></label>${seg("kelamin", [["Perempuan", t("f")], ["Laki-laki", t("m")]])}</div>
      <div class="field"><label>${t("td")} <span class="muted">(${t("opt")})</span></label><div class="unit"><input id="f_td" value="${esc(f.td)}" placeholder="${t("tdPh")}" inputmode="numeric"><span>mmHg</span></div></div>
      <div class="field"><label>${t("bb")} <span class="muted">(${t("opt")})</span></label><div class="unit"><input id="f_bb" type="number" min="20" max="250" step="0.1" value="${esc(f.bb)}" placeholder="60"><span>${t("bbU")}</span></div></div>
      <div class="field" style="grid-column:1/-1"><label>${t("pos")}</label>${seg("posisi", [["berdiri", t("posB")], ["duduk", t("posD")]])}<p class="muted" style="margin-top:6px">${t("posNote")}</p></div></div>
      <div class="row"><button class="btn ghost" id="f_back" style="${sbtn}">${t("back")}</button>
        <button class="btn right" id="go" ${st.busy ? "disabled" : ""}>${edit ? t("save") : st.busy ? t("starting") : t("goScan")}</button></div></div>`;
  const keep = () => { f.nama = $("#f_nama").value; f.usia = $("#f_usia").value; f.td = $("#f_td").value; f.bb = $("#f_bb").value; };
  m.querySelectorAll("[data-kelamin]").forEach((b) => (b.onclick = () => { keep(); f.kelamin = b.dataset.kelamin; render(); }));
  m.querySelectorAll("[data-posisi]").forEach((b) => (b.onclick = () => { keep(); f.posisi = b.dataset.posisi; render(); }));
  if (!edit) { paintGate(); invoke("data_dir_status").then((r) => { dd = r; paintGate(); }).catch(() => {}); }
  $("#f_back").onclick = () => { keep(); if (edit) { st.screen = st.editBack; if (st.screen === "inlow") openInlow(); if (st.screen === "notes") { openNotes(); return; } render(); } else toHub(); };
  $("#go").onclick = async () => {
    keep();
    const age = f.usia === "" ? NaN : Number(f.usia), bb = f.bb === "" ? null : Number(f.bb), td = f.td.replace(/\s/g, "");
    const bad = !f.nama.trim() ? "e_C_NAME" : !(Number.isInteger(age) && age >= 0 && age <= 120) ? "e_C_AGE" : !f.kelamin ? "e_C_SEX"
      : td && !/^\d{2,3}\/\d{2,3}$/.test(td) ? "e_C_TD" : bb != null && !(bb >= 20 && bb <= 250) ? "e_C_BB" : null;
    if (bad) { $("#ferr").innerHTML = `<div class="err">${t(bad)}</div>`; return; }
    const form = { nama: f.nama, usia: age, kelamin: f.kelamin, td, bb, posisi: f.posisi };
    if (edit) {
      try { st.pat = takeWarn(await invoke("update_patient", { key: st.pat.key, form })); st.error = ""; st.screen = st.editBack; if (st.screen === "inlow") openInlow(); if (st.screen === "notes") { openNotes(); return; } }
      catch (e) { st.error = e; }
      render(); return;
    }
    st.busy = true; st.error = ""; render();
    try {
      st.pat = await invoke("start_patient", { form });
      localStorage.setItem("lastPosisi", f.posisi);
      st.side = "left"; st.scan = null; st.screen = "scan";
      sc.lock = false; if (sc.test && !sc.test.ok) sc.test = null;
    } catch (e) { st.error = e; noteScanError(e); }
    st.busy = false; render(); pollScanner();
  };
}

function scanScreen(m) {
  const side = st.side, stand = st.pat.patient.posisi === "berdiri";
  const sub = t((stand ? "subStand" : "subSit") + (side === "left" ? "L" : "R"));
  m.innerHTML = `<h1>${t("scan")} · ${sideName(side)}</h1><p class="sub">${sub}</p><div class="grid2"><div class="card"><div class="scanwin">${guideFoot(side)}</div></div>
    <div class="card"><div class="pname">${String(st.pat.patient.no).padStart(3, "0")} · ${esc(st.pat.patient.nama)}</div>${errBox()}<div id="sstate" style="margin:20px 0 14px;color:var(--mute)">${t("ready")}</div><div class="bar"><b id="sbar"></b></div>
    <div class="row" style="margin-top:26px"><button class="btn" id="sgo">${st.error ? t("retry") : t("doScan")}</button></div>
    <div class="row" style="margin-top:22px"><button class="btn bad" id="cancel" style="padding:10px 18px;font-size:15px">${t("cancelP")}</button></div></div></div>`;
  armed($("#cancel"), t("cancelP"), t("cancelSure"), cancelPatient);
  $("#sgo").onclick = async () => {
    st.error = ""; $("#sgo").disabled = true; $("#sstate").textContent = t("scanning"); $("#cancel").disabled = true;
    let p = 0; const iv = setInterval(() => { p += (95 - p) * 0.04; $("#sbar").style.width = p + "%"; }, 400);
    try {
      st.scanning = true; steps();
      st.scan = await invoke("scan_foot", { key: st.pat.key, side });
      clearInterval(iv); st.screen = "review"; sc.lock = false; if (sc.test && !sc.test.ok) sc.test = null;
    } catch (e) { clearInterval(iv); st.error = e; noteScanError(e); }
    st.scanning = false;
    render();
  };
}

function turnScreen(m) {
  m.innerHTML = `<h1>${t("turn")}</h1><p class="sub">${t("turnSub")}</p>${noticeBox()}${warnBox()}${errBox()}<div class="card"><div class="row" style="gap:36px;flex-wrap:nowrap">
    <div style="flex:0 0 260px">${turnPic}</div><div style="flex:1"><div class="pname">${String(st.pat.patient.no).padStart(3, "0")} · ${esc(st.pat.patient.nama)}</div>
    <p style="font-size:20px;line-height:1.5;margin:14px 0 26px">${t("turnSub")}</p><button class="btn" id="tgo">${t("turnGo")}</button>
    <div class="row" style="margin-top:22px"><button class="btn bad" id="cancel" style="padding:10px 18px;font-size:15px">${t("cancelP")}</button></div></div></div></div>`;
  armed($("#cancel"), t("cancelP"), t("cancelSure"), cancelPatient);
  $("#tgo").onclick = () => { st.side = "right"; st.scan = null; st.error = ""; st.screen = "scan"; render(); };
}

function reviewScreen(m) {
  const side = st.side, s = st.scan, w = s.warnings;
  const noFoot = w.includes("NO_FOOT"), edge = w.includes("EDGE"), ok = !w.length;
  const warnText = [noFoot ? t("wNoFoot") : "", edge ? t("wEdge") : "", w.includes("ODD_SHAPE") ? t("wOdd") : "", w.includes("TWO_OBJECTS") ? t("wTwo") : ""].filter(Boolean).join(" ");
  // outline switch: ON every time a new scan is reviewed
  const canTog = !noFoot && s.plain;
  m.innerHTML = `<h1>${t("review")} · ${sideName(side)}</h1><p class="sub">${t("reviewSub")}</p>${errBox()}<div class="grid2"><div class="card"><div class="scanwin" style="background:#10151f"><img id="rvimg" src="${s.preview}" alt="">
      ${canTog ? `<label class="tog"><input type="checkbox" id="rvtog" checked> ${X[L].outline}</label>` : ""}</div></div>
    <div class="card" id="rvcard"><ul class="checks"><li><span class="dot ${noFoot ? "bad" : ""}"></span>${t("chkFound")}</li><li><span class="dot ${edge || noFoot ? "bad" : ""}"></span>${t("chkInside")}</li></ul>
    ${ok ? "" : `<div class="warn">${warnText}</div>`}
    <div class="row" style="margin-top:24px"><button class="btn ${ok ? "bad" : ""}" id="retake">↺ ${t("retake")}</button>
    <button class="btn ${ok ? "ok" : "ghost"} right" id="accept">${ok ? "✓ " + t("accept") : t("acceptAnyway")}</button></div>
    ${noFoot ? "" : metricsBlock(s.measurements)}
    <div class="row" style="margin-top:22px"><button class="btn bad" id="cancel" style="padding:10px 18px;font-size:15px">${t("cancelP")}</button></div></div></div>`;
  armed($("#cancel"), t("cancelP"), t("cancelSure"), cancelPatient);
  if ($("#rvtog")) $("#rvtog").onchange = (e) => { $("#rvimg").src = e.target.checked ? s.preview : s.plain; };
  if (s.measurements) wireInfo($("#rvcard"), { img: s.plain || s.preview, w: s.preview_w, h: s.preview_h, down: s.width_px / s.preview_w, m: s.measurements });
  $("#retake").onclick = async () => {
    try { await invoke("retake_scan", { key: st.pat.key, side }); st.scan = null; st.screen = "scan"; st.error = ""; } catch (e) { st.error = e; }
    render();
  };
  $("#accept").onclick = async () => {
    $("#accept").disabled = true; $("#accept").textContent = t("saving"); $("#retake").disabled = true;
    try {
      st.pat = takeWarn(await invoke("accept_scan", { key: st.pat.key, side, seconds: s.seconds, widthPx: s.width_px, heightPx: s.height_px, warnings: w }));
      const outline = s.preview; st.scan = null; st.error = "";
      if (s.measurements) { await openCheck(side, outline); return; }
      afterFoot();
    } catch (e) { st.error = e; }
    render();
  };
}

// ---- point check ("Periksa titik"): right after each foot is accepted ----
// Clarke's three points (orange) and the AI heel-centre / 2nd-toe points (white) are dragged on the
// scan; "Titik sudah benar" saves them (confirm_points), "Lewati" records a reason (skip_points).
// Only a confirmed check gives Clarke's angle its flat / normal / high label (in Rust and Excel).
const ck = { side: null, view: null, outline: null, pts: null, auto: null, layers: null, skipOpen: false, drag: null, back: null };
const clarkeLabel = (deg) => (deg < 31 ? "datar" : deg <= 41 ? "normal" : "tinggi"); // same as measure::clarke_label
function clarkeDeg(P, sc) {
  const mm = (q) => [q[0] * sc.x_mm_per_px, q[1] * sc.y_mm_per_px];
  const f = mm(P.fore), h = mm(P.heel), a = mm(P.arch), u = [h[0] - f[0], h[1] - f[1]], v = [a[0] - f[0], a[1] - f[1]];
  const c = (u[0] * v[0] + u[1] * v[1]) / Math.max(1e-6, Math.hypot(...u) * Math.hypot(...v));
  return (Math.acos(Math.max(-1, Math.min(1, c))) * 180) / Math.PI;
}
// back = optional function to run after confirm/skip instead of moving on in the patient flow
async function openCheck(side, outline, back) {
  try {
    const v = await invoke("get_scan_view", { key: st.pat.key, side });
    const m = v.measurements;
    if (!m) { back ? back() : afterFoot(); return; }
    // rough fallback if the backend had no Clarke guess: tangent from toe base to heel end, arch midway
    const g = m.clarke_pts ?? { fore: m.toe_base, heel: m.heel_end, arch: [(m.toe_base[0] + m.heel_end[0]) / 2, (m.toe_base[1] + m.heel_end[1]) / 2] };
    const auto = { fore: [...g.fore], heel: [...g.heel], arch: [...g.arch], hc: [...m.heel_centre], toe: [...m.toe_tip] };
    const last = v.check?.status === "diperiksa" ? v.check.clarke_pts : null;
    Object.assign(ck, {
      side, view: v, outline, auto, skipOpen: false, back: back ?? null, roughArmed: false,
      pts: last ? { fore: [...last.fore], heel: [...last.heel], arch: [...last.arch], hc: [...m.heel_centre], toe: [...m.toe_tip] } : JSON.parse(JSON.stringify(auto)),
      layers: { outline: !!outline, ai: false, clarke: true, contact: false, marks: false },
    });
    st.side = side; st.screen = "check"; st.error = "";
  } catch (e) { st.error = e; }
  render();
}
// where the patient goes once a foot is accepted and checked (or skipped)
function afterFoot() {
  const next = st.pat.next_side;
  if (next === "right") { st.side = "right"; st.screen = "turn"; }
  else if (next === "left") { st.side = "left"; st.screen = "scan"; }
  else openInlow();
  render();
}
const noticeBox = () => { const n = st.notice; st.notice = null; return n ? `<div class="okbox">${n}</div>` : ""; };
const degText = (d) => num(d, 1) + "°";

function checkSvg() {
  const v = ck.view, m = v.measurements, d = v.down, P = ck.pts, Y = ck.layers;
  const q = (p) => [p[0] / d, p[1] / d];
  const [F, Hl, A, HC, J] = [q(P.fore), q(P.heel), q(P.arch), q(P.hc), q(P.toe)];
  let g = "";
  if (Y.contact && v.contact_preview) g += `<image href="${v.contact_preview}" width="${v.preview_w}" height="${v.preview_h}" style="mix-blend-mode:screen" opacity=".95"/>`;
  if (Y.ai) { // Cavanagh thirds from the automatic measurement (they move only after confirming)
    const E = q(m.heel_end), T0 = q(m.toe_tip), l = Math.hypot(T0[0] - E[0], T0[1] - E[1]) || 1, a = [(T0[0] - E[0]) / l, (T0[1] - E[1]) / l], b = [-a[1], a[0]];
    const ppm = 1 / (m.scale.x_mm_per_px * d), half = (m.fw_mm / 2) * 1.15 * ppm, Lt = m.toeless_frac * m.fl_mm * ppm;
    for (let i = 0; i <= 3; i++) {
      const C = [E[0] + a[0] * Lt * i / 3, E[1] + a[1] * Lt * i / 3];
      g += `<line x1="${C[0] - b[0] * half}" y1="${C[1] - b[1] * half}" x2="${C[0] + b[0] * half}" y2="${C[1] + b[1] * half}" stroke="${i == 3 ? "#ffd166" : "#fff"}" stroke-width="${i == 3 ? 3 : 1.5}" stroke-dasharray="${i == 3 ? "8 5" : "3 5"}" opacity=".85"/>`;
    }
  }
  if (Y.marks) (st.pat.patient.marks ?? []).filter((k) => k.side === ck.side).forEach((k) => {
    g += `<circle cx="${k.x / d}" cy="${k.y / d}" r="${k.r / d}" fill="none" stroke="#ff8fab" stroke-width="3" stroke-dasharray="5 4"/><text x="${k.x / d}" y="${k.y / d - k.r / d - 6}" text-anchor="middle" class="mklab" style="font-size:16px">${esc(k.label)}</text>`;
  });
  g += `<line x1="${HC[0]}" y1="${HC[1]}" x2="${J[0]}" y2="${J[1]}" stroke="#fff" stroke-width="2" stroke-dasharray="6 4" opacity=".85"/>`;
  if (Y.clarke) {
    const u = [Hl[0] - F[0], Hl[1] - F[1]], lu = Math.hypot(...u) || 1, w = [A[0] - F[0], A[1] - F[1]], lw = Math.hypot(...w) || 1, r = 46;
    const e1 = [F[0] - (u[0] / lu) * 60, F[1] - (u[1] / lu) * 60], e2 = [Hl[0] + (u[0] / lu) * 40, Hl[1] + (u[1] / lu) * 40];
    const p1 = [F[0] + (u[0] / lu) * r, F[1] + (u[1] / lu) * r], p2 = [F[0] + (w[0] / lw) * r, F[1] + (w[1] / lw) * r];
    const sweep = u[0] * w[1] - u[1] * w[0] > 0 ? 1 : 0;
    g += `<line x1="${e1[0]}" y1="${e1[1]}" x2="${e2[0]}" y2="${e2[1]}" stroke="#ffb703" stroke-width="3"/><line x1="${F[0]}" y1="${F[1]}" x2="${A[0]}" y2="${A[1]}" stroke="#ffb703" stroke-width="3"/>
      <path d="M${p1[0]} ${p1[1]} A${r} ${r} 0 0 ${sweep} ${p2[0]} ${p2[1]}" stroke="#ffb703" stroke-width="2.5" fill="none"/>`;
  }
  const dot = (id, p, col, lab) => { const [lx, ly, an] = labelAt(id, p, HC, J);
    return `<g class="hdl" data-p="${id}"><circle cx="${p[0]}" cy="${p[1]}" r="13" fill="${col}" fill-opacity=".35" stroke="${col}" stroke-width="3.5"/><circle cx="${p[0]}" cy="${p[1]}" r="2.5" fill="${col}"/><circle cx="${p[0]}" cy="${p[1]}" r="26" fill="transparent"/>
    <text x="${lx}" y="${ly}" text-anchor="${an}" class="hlab">${esc(lab)}</text></g>`; };
  g += dot("fore", F, "#ffb703", t("lFore")) + dot("heel", Hl, "#ffb703", t("lHeel")) + dot("arch", A, "#ffb703", t("lArch")) + dot("hc", HC, "#ffffff", t("lHc")) + dot("toe", J, "#ffffff", t("lToe"));
  const base = Y.outline && ck.outline ? ck.outline : v.preview;
  // magnifier: the same picture 3x, clipped to a circle, shown while a point is held
  g += `<g id="lens" style="display:none;pointer-events:none"><clipPath id="lensclip"><circle id="lensc" r="95"/></clipPath><circle id="lensbg" r="97" fill="#000"/>
    <g clip-path="url(#lensclip)"><g id="lensimg"><image href="${base}" width="${v.preview_w}" height="${v.preview_h}"/></g></g>
    <circle id="lensring" r="95" fill="none" stroke="#fff" stroke-width="4"/><path id="lensx" stroke="#ffb703" stroke-width="2" fill="none"/></g>`;
  return `<svg id="cksvg" viewBox="0 0 ${v.preview_w} ${v.preview_h}" style="touch-action:none"><image href="${base}" width="${v.preview_w}" height="${v.preview_h}"/>${g}</svg>`;
}
// the orange points still sit on the app's rough guess (not the contact construction)
function roughUntouched() {
  if (ck.view.measurements.clarke_source === "kontak") return false;
  const sc = ck.view.measurements.scale, dmm = (a, b) => Math.hypot((a[0] - b[0]) * sc.x_mm_per_px, (a[1] - b[1]) * sc.y_mm_per_px);
  return ["fore", "heel", "arch"].every((k) => dmm(ck.pts[k], ck.auto[k]) <= 0.5);
}
function movedCount() {
  const sc = ck.view.measurements.scale, dmm = (a, b) => Math.hypot((a[0] - b[0]) * sc.x_mm_per_px, (a[1] - b[1]) * sc.y_mm_per_px);
  return ["fore", "heel", "arch", "hc", "toe"].filter((k) => dmm(ck.pts[k], ck.auto[k]) > 0.5).length;
}
function paintCheckNums() {
  const m = ck.view.measurements, deg = clarkeDeg(ck.pts, m.scale), n = movedCount();
  $("#cknums").innerHTML = `<div class="ckbig"><div class="ml">${t("clarke")}</div><div class="mv">${degText(deg)}</div>
      <div class="muted">${m.clarke_source === "kontak" ? t("guessK") : t("guessP")}</div><div class="muted" style="margin-top:4px">${t("noLabel")}</div></div>
    <div class="kv"><span>${t("aiNow")}</span><b>${num(m.ai, 3)}</b></div><p class="muted" style="margin:4px 0 0">${t("aiLater")}</p>
    ${n ? `<span class="chip warn" style="margin-top:10px">${n} ${t("moved")}</span>` : ""}`;
}
function wireCheckDrag() {
  const svg = $("#cksvg"), v = ck.view, d = v.down;
  const toSvg = (e) => { const q = svg.createSVGPoint(); q.x = e.clientX; q.y = e.clientY; return q.matrixTransform(svg.getScreenCTM().inverse()); };
  const lens = (p) => {
    const L0 = $("#lens"); if (!p) { L0.style.display = "none"; return; }
    const Z = 3, R = 95, up = p[1] > R * 2.4 ? -1 : 1, c = [Math.min(Math.max(p[0], R + 4), v.preview_w - R - 4), p[1] + up * R * 1.55];
    L0.style.display = "";
    ["#lensc", "#lensbg", "#lensring"].forEach((s) => { $(s).setAttribute("cx", c[0]); $(s).setAttribute("cy", c[1]); });
    $("#lensimg").setAttribute("transform", `translate(${c[0]} ${c[1]}) scale(${Z}) translate(${-p[0]} ${-p[1]})`);
    $("#lensx").setAttribute("d", `M${c[0] - 16} ${c[1]}H${c[0] + 16}M${c[0]} ${c[1] - 16}V${c[1] + 16}`);
  };
  svg.querySelectorAll(".hdl").forEach((h) => {
    h.onpointerdown = (e) => { e.preventDefault(); ck.drag = h.dataset.p; h.setPointerCapture(e.pointerId); h.classList.add("on"); lens(ck.pts[ck.drag].map((x) => x / d)); };
    h.onpointermove = (e) => {
      if (ck.drag !== h.dataset.p) return;
      const q = toSvg(e), p = [Math.min(Math.max(q.x, 0), v.preview_w), Math.min(Math.max(q.y, 0), v.preview_h)];
      ck.pts[ck.drag] = [p[0] * d, p[1] * d];
      moveHandle(h, p); lens(p); paintCheckLines(); paintCheckNums();
      if (ck.roughArmed && !roughUntouched()) { ck.roughArmed = false; const okb = $("#ck_ok"); okb.textContent = t("confirm"); okb.parentElement.nextElementSibling?.classList.contains("warn") && okb.parentElement.nextElementSibling.remove(); }
    };
    h.onpointerup = h.onpointercancel = () => { if (ck.drag !== h.dataset.p) return; ck.drag = null; h.classList.remove("on"); lens(null); };
  });
}
// label beside a handle: Clarke points on the side away from the foot axis, axis points on the
// other side, below the point when it sits at the top edge
function labelAt(id, p, HC, J) {
  const away = (J[0] - HC[0]) * (p[1] - HC[1]) - (J[1] - HC[1]) * (p[0] - HC[0]) > 0 ? -1 : 1;
  const dir = ["fore", "heel", "arch"].includes(id) ? -away : 1;
  return [p[0] + dir * 18, p[1] < 34 ? p[1] + 34 : p[1] - 14, dir > 0 ? "start" : "end"];
}
// while dragging only the handle and the lines move; a full redraw would drop the pointer capture
function moveHandle(h, p) {
  h.querySelectorAll("circle").forEach((c) => { c.setAttribute("cx", p[0]); c.setAttribute("cy", p[1]); });
  const d = ck.view.down, [lx, ly, an] = labelAt(h.dataset.p, p, [ck.pts.hc[0] / d, ck.pts.hc[1] / d], [ck.pts.toe[0] / d, ck.pts.toe[1] / d]);
  const tx = h.querySelector("text"); tx.setAttribute("x", lx); tx.setAttribute("y", ly); tx.setAttribute("text-anchor", an);
}
function paintCheckLines() {
  const tmp = document.createElement("div"); tmp.innerHTML = checkSvg();
  const fresh = tmp.querySelector("svg"), svg = $("#cksvg");
  // replace everything except the handles and the lens (they are live)
  [...svg.children].forEach((c) => { if (!c.classList.contains("hdl") && c.id !== "lens") c.remove(); });
  const first = svg.querySelector(".hdl");
  [...fresh.children].forEach((c) => { if (!c.classList.contains("hdl") && c.id !== "lens") svg.insertBefore(c, first); });
}
function checkScreen(m) {
  const v = ck.view, Y = ck.layers;
  const tog = (k, lab, ok = true) => (ok ? `<label class="ly"><input type="checkbox" data-ly="${k}" ${Y[k] ? "checked" : ""}> ${lab}</label>` : "");
  m.innerHTML = `<h1>${t("chk")} · ${sideName(ck.side)}</h1><p class="sub">${t("chkSub")}</p>${warnBox()}${errBox()}
    <div class="ckgrid"><div class="card" style="padding:16px"><div class="ckwin" id="ckimg">${checkSvg()}</div>
      <div class="lys">${tog("outline", t("lyOutline"), !!ck.outline)}${tog("ai", t("lyAI"))}${tog("clarke", t("lyClarke"))}${tog("contact", t("lyContact"), !!v.contact_preview)}${tog("marks", t("lyMarks"), !!st.pat.patient.marks?.some((k) => k.side === ck.side))}</div>
      ${Y.contact ? `<p class="muted" style="margin-top:6px">${t("contactCap")}</p>` : ""}</div>
    <div class="card"><p class="cklegend"><span class="sw o"></span>${t("chkClarke")}</p><p class="cklegend"><span class="sw w"></span>${t("chkAxis")}</p>
      <div id="cknums"></div>
      <div class="row" style="margin-top:18px"><button class="btn ghost" id="ck_reset" style="${sbtn}">↺ ${t("reset")}</button></div>
      <div class="row" style="margin-top:18px"><button class="btn ghost" id="ck_skip" style="${sbtn}">${t("skip")}</button><button class="btn ok right" id="ck_ok">${ck.roughArmed ? t("roughOk") : t("confirm")}</button></div>
      ${ck.roughArmed ? `<div class="warn">${t("roughWarn")}</div>` : ""}
      ${ck.back ? `<div class="row" style="margin-top:18px"><button class="btn ghost" id="ck_back" style="${sbtn}">${t("back")}</button></div>` : ""}
      ${ck.skipOpen ? `<div class="skipbox"><div style="font-weight:700;margin-bottom:10px">${t("skipQ")}</div><div class="row">${Object.entries(t("skipR")).map(([k, lab]) => `<button class="btn ghost" data-skip="${k}" style="${sbtn}">${lab}</button>`).join("")}</div><p class="muted" style="margin-top:10px">${t("skipNote")}</p></div>` : ""}</div></div>`;
  paintCheckNums(); wireCheckDrag();
  m.querySelectorAll("[data-ly]").forEach((c) => (c.onchange = () => { Y[c.dataset.ly] = c.checked; render(); }));
  $("#ck_reset").onclick = () => { ck.pts = JSON.parse(JSON.stringify(ck.auto)); ck.roughArmed = false; render(); };
  $("#ck_skip").onclick = () => { ck.skipOpen = !ck.skipOpen; render(); };
  if ($("#ck_back")) $("#ck_back").onclick = () => { const b = ck.back; ck.back = null; b(); };
  const done = (info, text) => {
    st.pat = takeWarn(info); st.notice = text;
    if (ck.back) { const b = ck.back; ck.back = null; b(); return; }
    afterFoot();
  };
  m.querySelectorAll("[data-skip]").forEach((b) => (b.onclick = async () => {
    m.querySelectorAll("button").forEach((x) => (x.disabled = true));
    try { done(await invoke("skip_points", { key: st.pat.key, side: ck.side, reason: b.dataset.skip }), `${sideName(ck.side)}: ${t("okSkip")} (${esc(t("skipR")[b.dataset.skip])})`); }
    catch (e) { st.error = e; render(); }
  }));
  $("#ck_ok").onclick = async () => {
    if (roughUntouched() && !ck.roughArmed) { ck.roughArmed = true; render(); return; }
    const b = $("#ck_ok"); b.disabled = true; b.textContent = t("saving");
    const P = ck.pts, deg = clarkeDeg(P, v.measurements.scale);
    try {
      done(await invoke("confirm_points", { key: st.pat.key, side: ck.side, heel: P.hc, toe: P.toe, clarke: { fore: P.fore, heel: P.heel, arch: P.arch } }),
        `✓ ${sideName(ck.side)} ${t("okChk")} · ${t("clarke")} ${degText(deg)} (${t("lbl")[clarkeLabel(deg)]})`);
    } catch (e) { st.error = e; render(); }
  };
}

function inlowScreen(m) {
  const I = st.inl, spec = st.spec;
  const feet = [["kanan", t("kanan")], ["kiri", t("kiri")]];
  let lastGroup = "";
  const rows = spec.items.map((it) => {
    const head = it.group !== lastGroup ? `<div class="igroup">${esc((lastGroup = it.group))}</div>` : "";
    const cell = (foot) => `<div class="iopts">${it.options.map(([pts, txt]) => `<button type="button" class="iopt ${I[foot].items[it.key] === pts ? "on" : ""}" data-f="${foot}" data-k="${it.key}" data-v="${pts}"><span>${esc(txt)}</span><em>${pts}</em></button>`).join("")}</div>`;
    const qs = it.key === "tanya" ? `<details class="iq" ${I.kanan.questions.some((q) => q != null) || I.kiri.questions.some((q) => q != null) ? "open" : ""}><summary>${t("qOpen")}</summary>
        ${spec.questions.map((q, qi) => `<div class="iqrow"><span>${esc(q)}</span>${feet.map(([f]) => `<div class="seg mini">${[[true, t("yes")], [false, t("no")]].map(([v, lab]) => `<button type="button" class="${I[f].questions[qi] === v ? "on" : ""}" data-q="${qi}" data-qf="${f}" data-qv="${v}">${lab}</button>`).join("")}</div>`).join("")}</div>`).join("")}
        <p class="muted" style="margin-top:6px">${t("qNote")}</p></details>` : "";
    return `${head}<div class="irow"><div class="ilab">${esc(it.label)}</div>${cell("kanan")}${cell("kiri")}</div>${qs}`;
  }).join("");
  const sum = (foot) => { const f = I[foot], n = footFilled(f), tot = footTotal(f), r = tot != null ? riskOf(tot) : null;
    return `<div class="isum"><b>${t(foot)}</b> · ${n}/${spec.items.length} ${t("filled")}${r ? ` · ${t("total")} <b>${tot}</b> · <span class="chip risk${r[1]}">${r[1]} ${esc(r[2])}</span>` : ""}</div>`; };
  const complete = footTotal(I.kanan) != null && footTotal(I.kiri) != null;
  m.innerHTML = `<h1>${t("inlow")}</h1><p class="sub">${t("inlowSub")}</p>${noticeBox()}${warnBox()}${errBox()}
    <div class="card"><div class="irow ihead"><div></div><div>${t("kanan")}</div><div>${t("kiri")}</div></div>${rows}</div>
    <div class="ibar"><button class="btn ghost" id="to_list" style="${sbtn}">${t("toList")}</button>${sum("kanan")}${sum("kiri")}${complete ? "" : `<span class="chip warn">${t("inlowNo")}</span>`}<button class="btn right" id="i_next" style="${sbtn}">${t("toNotes")}</button></div>
    <p class="muted" style="margin-top:12px">${esc(spec.notes)}</p>`;
  m.querySelectorAll(".iopt").forEach((b) => (b.onclick = () => {
    const f = I[b.dataset.f], k = b.dataset.k, v = Number(b.dataset.v);
    if (f.items[k] === v) delete f.items[k]; else f.items[k] = v;
    if (k === "tanya") f.questions = [null, null, null, null]; // a direct choice replaces the question answers
    st.inlDirty = true; render();
  }));
  m.querySelectorAll("[data-q]").forEach((b) => (b.onclick = () => {
    const f = I[b.dataset.qf], qi = Number(b.dataset.q), v = b.dataset.qv === "true";
    f.questions[qi] = f.questions[qi] === v ? null : v;
    if (f.questions.every((q) => q != null)) f.items.tanya = f.questions.some((q) => q) ? 2 : 0;
    st.inlDirty = true; render();
  }));
  $("#i_next").onclick = async () => { if (await saveDrafts()) { openNotes(); } };
  $("#to_list").onclick = async () => { if (await saveDrafts()) toHub(); };
}

const MARK_R = 120; // full-resolution pixels, about 10 mm
const MARK_COLORS = { kalus: "#ffb703", luka: "#ff4d6d" };
function paintMarks() {
  ["left", "right"].forEach((side) => {
    const box = $("#mk_" + side); if (!box) return;
    const v = st.views[st.pat.key + "|" + side];
    if (!hasScan(st.pat.patient, side)) { box.innerHTML = `<div class="muted" style="padding:40px;text-align:center">${t("noScan")}</div>`; return; }
    if (!v) { box.innerHTML = ""; return; }
    const ms = st.nt.marks.map((mk, i) => [mk, i]).filter(([mk]) => mk.side === side);
    box.innerHTML = `<svg viewBox="0 0 ${v.preview_w} ${v.preview_h}" class="mksvg ${st.nt.marking ? "on" : ""}"><image href="${v.preview}" width="${v.preview_w}" height="${v.preview_h}"/>
      ${ms.map(([mk, i]) => { const c = MARK_COLORS[mk.label.toLowerCase()] ?? "#5aa0ff", d = v.down;
        return `<g data-mi="${i}" class="mk"><circle cx="${mk.x / d}" cy="${mk.y / d}" r="${mk.r / d}" fill="${c}33" stroke="${c}" stroke-width="3"/><text x="${mk.x / d}" y="${mk.y / d - mk.r / d - 6}" text-anchor="middle" class="mklab">${esc(mk.label)}</text></g>`; }).join("")}</svg>`;
    const svg = box.querySelector("svg");
    svg.onclick = (e) => {
      if (!st.nt.marking) return;
      const hit = e.target.closest("[data-mi]");
      if (hit) { st.nt.marks.splice(Number(hit.dataset.mi), 1); }
      else {
        const label = st.nt.label === "lain" ? st.nt.other.trim() || t("mLain") : st.nt.label;
        const q = svg.createSVGPoint(); q.x = e.clientX; q.y = e.clientY; const p = q.matrixTransform(svg.getScreenCTM().inverse());
        st.nt.marks.push({ side, x: Math.round(p.x * v.down), y: Math.round(p.y * v.down), r: MARK_R, label });
      }
      st.ntDirty = true; paintMarks(); paintMarkCount();
    };
  });
}
function paintMarkCount() { const el = $("#mkcount"); if (el) el.textContent = `${st.nt.marks.length} ${t("marks")}`; }
function notesScreen(m) {
  const N = st.nt;
  const lab = (k, txt) => `<button type="button" class="${N.label === k ? "on" : ""}" data-ml="${k}">${txt}</button>`;
  m.innerHTML = `<h1>${t("notes")}</h1><p class="sub">${t("notesSub")}</p>${warnBox()}${errBox()}
    <div class="grid2"><div class="field"><label>${t("nKiri")}</label><textarea id="n_kiri" rows="3">${esc(N.notes.kiri)}</textarea></div>
      <div class="field"><label>${t("nKanan")}</label><textarea id="n_kanan" rows="3">${esc(N.notes.kanan)}</textarea></div>
      <div class="field" style="grid-column:1/-1"><label>${t("nUmum")}</label><textarea id="n_umum" rows="2">${esc(N.notes.umum)}</textarea></div></div>
    <div class="card"><div class="row"><h2 style="font-size:20px">${t("mark")}</h2><span class="chip" id="mkcount"></span>
      <button class="btn ${N.marking ? "ok" : "ghost"} right" id="mk_tog" style="${sbtn}">${N.marking ? "✓ " + t("markOff") : t("mark")}</button></div>
      ${N.marking ? `<p class="muted" style="margin:10px 0">${t("markSub")}</p><div class="row"><label style="margin:0">${t("mLabel")}</label><div class="seg small" style="flex:none">${lab("kalus", t("mKalus"))}${lab("luka", t("mLuka"))}${lab("lain", t("mLain"))}</div>
        ${N.label === "lain" ? `<input id="mk_other" value="${esc(N.other)}" placeholder="${t("mLainPh")}" maxlength="40" style="width:240px">` : ""}</div>` : ""}
      <div class="grid2" style="margin-top:14px"><div><div class="muted" style="margin-bottom:6px">${t("left")}</div><div class="mkbox" id="mk_left"></div></div>
        <div><div class="muted" style="margin-bottom:6px">${t("right")}</div><div class="mkbox" id="mk_right"></div></div></div></div>
    <div class="row" style="margin-top:22px"><button class="btn ghost" id="to_list" style="${sbtn}">${t("toList")}</button><button class="btn right" id="n_done">${st.pat.next_side ? t("toScan") : t("finish")}</button></div>`;
  ["kiri", "kanan", "umum"].forEach((k) => ($("#n_" + k).oninput = () => { st.ntDirty = true; }));
  $("#mk_tog").onclick = () => { keepNotes(); N.marking = !N.marking; render(); };
  m.querySelectorAll("[data-ml]").forEach((b) => (b.onclick = () => { keepNotes(); N.label = b.dataset.ml; render(); }));
  if ($("#mk_other")) $("#mk_other").oninput = (e) => { N.other = e.target.value; };
  paintMarks(); paintMarkCount();
  $("#to_list").onclick = async () => { if (await saveDrafts()) toHub(); };
  $("#n_done").onclick = async () => {
    const b = $("#n_done"); b.disabled = true; b.textContent = t("finishing");
    if (!(await saveDrafts())) return;
    if (st.pat.next_side) { st.side = st.pat.next_side; st.screen = hasScan(st.pat.patient, "left") ? "turn" : "scan"; render(); return; }
    await finishPatient();
  };
}

function finishedScreen(m) {
  const p = st.pat.patient, row = st.sesi?.patients.find((r) => r.key === st.pat.key);
  const bkBad = bk?.enabled && (!bk.reachable || bk.last_err) ? `<div class="warn">${t("bkDone")}<br>${bkLine()}</div>` : "";
  m.innerHTML = `<h1>${t("done")}</h1><p class="sub">${t("doneSub")}</p>${warnBox()}${bkBad}${errBox()}<div class="card"><div class="row" style="gap:30px;flex-wrap:nowrap"><div class="big" style="color:var(--ok)">✓</div>
      <div style="flex:1"><div style="font-size:24px;font-weight:800">${String(p.no).padStart(3, "0")} · ${esc(p.nama)}</div>
      <div class="sub" style="margin:4px 0 0">${t("saved")} ${esc(dd?.path ? dd.path + "\\" + st.pat.key : st.pat.key)}</div>
      ${row ? `<div class="concbox"><div class="muted">${t("conc")}</div><div>${esc(row.kesimpulan)}</div></div>` : ""}
      ${inlowComplete(p.inlow) ? "" : `<span class="chip warn" style="margin-top:10px">${t("inlowNo")}</span>`}</div></div>
    <div class="row" style="margin-top:26px"><button class="btn ghost" id="d_det" style="${sbtn}">${t("resBtn")}</button><button class="btn ghost" id="d_hub" style="${sbtn}">${t("toHub")}</button>
      <button class="btn right" id="d_next">${t("nextP")}</button></div></div>`;
  $("#d_det").onclick = () => openResearch(st.pat.key, "finished");
  $("#d_hub").onclick = () => toHub();
  $("#d_next").onclick = async () => { await loadSesi(); if (hubScreen() !== "sesi") { st.screen = hubScreen(); render(); return; } openForm(false); };
}

function setupScreen(m) {
  const c = st.settings;
  const sv = (st.setup ??= { dir: c.data_dir, bk: c.backup_dir ?? "", station: c.station, editCode: false });
  const browse = (id) => `<button class="btn ghost" id="${id}" style="padding:12px 18px;font-size:16px">${t("browse")}</button>`;
  m.innerHTML = `<h1>${t("setupT")}</h1><p class="sub">${t("setupSub")}</p>${errBox()}<div class="card" style="max-width:820px">
    <label>📁 ${t("setupData")} <span class="muted">(${t("required")})</span></label><div class="row"><input id="u_dir" value="${esc(sv.dir)}" style="flex:1">${browse("u_dir_b")}</div>
    <p class="muted" style="margin:6px 0 22px">${t("setupDataNote")}</p>
    <label>${t("setupBk")}</label><div class="row"><input id="u_bk" value="${esc(sv.bk)}" placeholder="E:\\SmartFoot_Backup" style="flex:1">${browse("u_bk_b")}</div>
    <p class="muted" style="margin:6px 0 22px">${t("setupBkNote")}</p>
    <div class="muted">${t("setupCode")}: <b style="color:var(--ink)">${esc(sv.station)}</b>${sv.editCode ? "" : ` · <button class="linkbtn" id="u_code" type="button">${t("ddChange")}</button>`}</div>
    ${sv.editCode ? `<div class="row" style="margin-top:8px"><input id="u_station" value="${esc(sv.station)}" maxlength="4" style="width:130px;text-transform:uppercase"><span class="muted" style="flex:1">${t("stationNote")}</span></div>` : ""}
    <div class="row" style="margin-top:26px"><button class="btn right" id="u_go">${t("setupGo")}</button></div></div>`;
  const keep = () => { sv.dir = $("#u_dir").value; sv.bk = $("#u_bk").value; if ($("#u_station")) sv.station = $("#u_station").value; };
  [["u_dir", "u_dir_b"], ["u_bk", "u_bk_b"]].forEach(([inp, btn]) => {
    $("#" + btn).onclick = async () => {
      let picked = null;
      try { picked = await invoke("pick_folder", { start: $("#" + inp).value }); } catch (e) { st.error = e; keep(); render(); return; }
      if (picked) { $("#" + inp).value = picked; keep(); }
    };
  });
  if ($("#u_code")) $("#u_code").onclick = () => { keep(); sv.editCode = true; render(); };
  $("#u_go").onclick = async () => {
    keep(); $("#u_go").disabled = true; $("#u_go").textContent = t("setupSaving");
    try {
      st.settings = await invoke("set_settings", { new: { ...c, data_dir: sv.dir.trim(), backup_dir: sv.bk.trim() || null, station: sv.station, setup_done: true } });
      st.error = ""; st.setup = null;
      await loadSesi(); st.screen = hubScreen(); await render(); pollScanner(); startTour(false);
    } catch (e) { st.error = e; render(); }
  };
}

function settingsScreen(m) {
  const c = st.settings;
  const val = (f) => st.draft[f] ?? FIELDS[f].get(c);
  const saveBtn = (f, id) => st.flash === f
    ? `<button class="btn ok" id="${id}" style="padding:12px 18px;font-size:16px">${t("savedBtn")}</button>`
    : `<button class="btn ghost" id="${id}" style="padding:12px 18px;font-size:16px" ${FIELDS[f].same(val(f), c) ? "disabled" : ""}>${t("save")}</button>`;
  const browseBtn = (id) => `<button class="btn ghost" id="${id}" style="padding:12px 18px;font-size:16px">${t("browse")}</button>`;
  const fieldErr = (f) => (st.ferr?.f === f ? `<div class="err" style="margin:8px 0 0">${errText(st.ferr.e)}</div>` : "");
  m.innerHTML = `<h1>${t("set")}</h1><p class="sub">${t("setSub")}</p>${errBox()}<div class="grid2">
    <div class="card"><label>${t("dataf")}</label><div class="row"><input id="s_dir" value="${esc(val("dir"))}" style="flex:1">${browseBtn("s_dir_b")}${saveBtn("dir", "s_save")}</div>${fieldErr("dir")}
      <label style="margin-top:22px">${t("station")}</label><div class="row"><input id="s_station" value="${esc(val("station"))}" maxlength="4" style="flex:1;text-transform:uppercase">${saveBtn("station", "s_stsave")}</div>${fieldErr("station")}<p class="muted" style="margin:6px 0 0">${t("stationNote")}</p>
      <label style="margin-top:22px">${t("scanner")}</label><div class="row"><select id="s_dev" style="flex:1"><option value="">${t("auto")}</option>${st.scanners.map((d) => `<option ${c.scanner == d ? "selected" : ""}>${esc(d)}</option>`).join("")}</select><button class="btn ghost" id="s_ref" style="padding:12px 18px;font-size:16px">${t("refresh")}</button></div>
      <label style="margin-top:22px">${t("bkLabel")}</label><div class="row"><input id="s_bk" value="${esc(val("bk"))}" placeholder="E:\\SmartFoot_Backup" style="flex:1">${browseBtn("s_bk_b")}${saveBtn("bk", "s_bksave")}</div>${fieldErr("bk")}
      <p class="muted" style="margin:6px 0 10px">${t("bkNote")}</p><div class="row"><span class="chip ${bk?.enabled && bk.reachable && !bk.last_err ? "ok" : "warn"}" id="bkstat" style="font-weight:600">${bkLine()}</span></div>
      <div class="row" style="margin-top:12px"><button class="btn ghost" id="s_bknow" style="padding:12px 18px;font-size:16px" ${bk?.enabled && bk.reachable ? "" : "disabled"}>${t("bkNow")}</button></div></div>
    <div><div class="card" id="scpanel"></div><div class="card" style="margin-top:18px"><h2 style="font-size:20px;margin-bottom:6px">${t("cal")}</h2><p style="line-height:1.5">${t("calNote")}</p>
      ${calibAnim()}
      <h4 style="margin:14px 0 4px;font-size:14px;text-transform:uppercase;letter-spacing:.5px;color:var(--mute)">${X[L].scaleH}</h4><p class="muted" style="line-height:1.5">${X[L].scale}</p>${rulerFig()}
      <p class="muted" style="line-height:1.5;margin-top:10px">${t("calCheck")}</p></div></div></div>
    <div class="row" style="margin-top:22px"><button class="btn ghost" id="back">${t("back")}</button><button class="btn right" id="gores">${t("resBtn")}</button></div>`;
  paintPanel();
  $("#gores").onclick = () => openResearch(null, "settings");
  // f = the text field being saved: feedback shows next to it ("✓ Tersimpan" or a red line)
  const persist = async (patch, f) => {
    st.ferr = null;
    try {
      st.settings = await invoke("set_settings", { new: { ...st.settings, ...patch } }); st.error = "";
      if (f) { delete st.draft[f]; st.flash = f; setTimeout(() => { if (st.flash === f) { st.flash = null; if (st.screen === "settings") render(); } }, 2200); }
    } catch (e) { if (f) st.ferr = { f, e }; else st.error = e; }
    render();
  };
  // folder picker: only fills the text box, Save still does the checks
  [["s_dir", "s_dir_b"], ["s_bk", "s_bk_b"]].forEach(([inp, btn]) => {
    $("#" + btn).onclick = async () => {
      let picked = null;
      try { picked = await invoke("pick_folder", { start: $("#" + inp).value }); } catch (e) { st.error = e; render(); return; }
      if (!picked) return; // cancelled
      const el = $("#" + inp); el.value = picked; el.dispatchEvent(new Event("input"));
    };
  });
  Object.entries({ dir: "s_dir", station: "s_station", bk: "s_bk" }).forEach(([f, id]) => {
    $("#" + id).oninput = (e) => {
      st.draft[f] = e.target.value; st.flash = null; st.ferr = null;
      const b = $("#" + { dir: "s_save", station: "s_stsave", bk: "s_bksave" }[f]);
      b.className = "btn ghost"; b.textContent = t("save"); b.toggleAttribute("disabled", FIELDS[f].same(e.target.value, st.settings));
      e.target.closest(".card").querySelectorAll(".err").forEach((x) => x.remove());
    };
  });
  // a new data folder means another set of sessions: reload the session state on the way back
  $("#s_save").onclick = () => persist({ data_dir: $("#s_dir").value.trim() }, "dir").then(async () => { if (!PATIENT_SCREENS.includes(st.prev)) { await loadSesi(); st.prev = hubScreen(); } });
  $("#s_stsave").onclick = () => persist({ station: $("#s_station").value }, "station");
  $("#s_bksave").onclick = () => persist({ backup_dir: $("#s_bk").value.trim() || null }, "bk").then(refreshBk);
  $("#s_bknow").onclick = async () => {
    const b = $("#s_bknow"); b.disabled = true; b.textContent = t("bkNowBusy");
    try { await invoke("backup_now"); st.error = ""; } catch (e) { st.error = e; }
    await refreshBk();
  };
  $("#s_dev").onchange = (e) => persist({ scanner: e.target.value || null });
  $("#s_ref").onclick = async () => { try { st.scanners = await invoke("list_scanners"); } catch (e) { st.error = e; } render(); pollScanner(); };
  $("#back").onclick = () => { st.screen = st.prev; st.error = ""; st.draft = {}; st.ferr = null; st.flash = null; render(); };
}

const SCREENS = { home: homeScreen, stale: staleScreen, sesi: sesiScreen, paused: pausedScreen, ended: endedScreen, pform: formScreen, scan: scanScreen, turn: turnScreen,
  review: reviewScreen, check: checkScreen, inlow: inlowScreen, notes: notesScreen, finished: finishedScreen, research: researchScreen, setup: setupScreen, settings: settingsScreen };

async function render() {
  steps();
  const m = $("#main");
  $("#teamBtn").textContent = t("team");
  const f = SCREENS[st.screen];
  if (f) f(m); else m.innerHTML = "";
  italicize($("aside")); italicize(m); paintChip();
}

$("#lang").onclick = () => { L = L == "id" ? "en" : "id"; localStorage.setItem("lang", L); render(); };
$("aside .row").insertAdjacentHTML("afterbegin", `<button class="lang" id="teamBtn"></button>`);
$("#teamBtn").onclick = async () => {
  if (st.screen === "settings") { st.screen = st.prev; }
  else {
    if (st.busy || st.scanning || st.screen === "boot") return;
    if (!(await saveDrafts())) return;
    st.prev = st.screen; st.screen = "settings"; st.error = ""; st.draft = {}; st.ferr = null; st.flash = null;
    try { st.settings = await invoke("get_settings"); try { bk = await invoke("backup_status"); } catch { bk = null; } st.scanners = await invoke("list_scanners").catch(() => []); } catch (e) { st.error = e; }
  }
  render();
};
$("#schip").onclick = () => { sc.phase = "checking"; paintChip(); pollScanner(); };
// first-run setup ("where is the data saved?") comes before anything else on a new laptop
(async () => {
  try { st.spec = await invoke("inlow_spec"); } catch (e) { st.error = e; }
  try { st.settings = await invoke("get_settings"); } catch (e) { st.error = e; }
  if (st.settings && !st.settings.setup_done) st.screen = "setup";
  else { await loadSesi(); st.screen = hubScreen(); }
  render();
})();
pollScanner();
setInterval(pollScanner, POLL_MS);

// ripple from the exact touch point on any button
document.addEventListener("pointerdown", (e) => {
  const b = e.target.closest(".btn, .seg button");
  if (!b || b.disabled) return;
  const r = b.getBoundingClientRect(), size = Math.max(r.width, r.height) * 2;
  const s = document.createElement("span");
  s.className = "ripple";
  s.style.cssText = `width:${size}px;height:${size}px;left:${e.clientX - r.left - size / 2}px;top:${e.clientY - r.top - size / 2}px`;
  b.appendChild(s);
  setTimeout(() => s.remove(), 600);
});

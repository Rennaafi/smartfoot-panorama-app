// SmartFoot front end. All hardware, file and image work happens in Rust; this file only draws
// screens and calls the commands in src-tauri/src/lib.rs.
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

const T = {
  id: {
    steps: ["Mulai", "Pindai", "Tinjau", "Selesai"], team: "⚙ Settings",
    start: "Sesi baru", startSub: "Isi data singkat pasien. Tidak perlu nama.",
    pid: "ID pasien", age: "Usia", sex: "Jenis kelamin", f: "Perempuan", m: "Laki-laki",
    op: "Inisial operator", opHint: "wajib diisi", note: "Catatan (opsional)", go: "Mulai pindai", starting: "Memeriksa scanner…",
    scan: "Pindai kaki", scanSub: "Letakkan kaki di atas kaca, tumit pada penahan. Diam sampai pemindaian selesai.",
    left: "Kaki kiri", right: "Kaki kanan", putaran: "Putaran", doScan: "Mulai pindai", scanning: "Memindai… jangan gerakkan kaki", ready: "Siap",
    review: "Tinjau hasil", reviewSub: "Garis biru = tepi kaki yang terdeteksi. Pastikan sesuai bentuk kaki.",
    chkFound: "Kaki terdeteksi", chkInside: "Kaki penuh di dalam area",
    wNoFoot: "Kaki tidak terdeteksi. Pastikan kaki menempel di kaca, lalu ulangi.",
    wEdge: "Kaki menyentuh tepi gambar. Geser kaki menjauhi tepi, lalu ulangi.",
    wOdd: "Bentuknya tidak seperti kaki (terlalu kecil, besar, atau memanjang). Pastikan yang dipindai adalah kaki.",
    wTwo: "Ada benda kedua di kaca. Hanya satu yang diberi garis. Singkirkan benda lain, lalu ulangi.",
    e_BAD_IMAGE: "Hasil pemindaian rusak atau terlalu kecil. Ulangi pemindaian.",
    dmg: "Ada sesi yang rusak atau tidak lengkap", dmgHint: "Jangan dihapus. Beri tahu tim.",
    e_BAD_ID: "ID pasien harus diawali huruf atau angka, maksimal 32 karakter, dan bukan nama khusus Windows (CON, NUL, COM1...).",
    e_BAD_OP: "Inisial operator harus diawali huruf atau angka, maksimal 8 karakter.", e_EMPTY_OP: "Isi inisial operator.",
    e_BAD_AGE: "Usia harus bilangan bulat 0–120.", e_BAD_SEX: "Pilih jenis kelamin.", e_BAD_DIR: "Folder harus berupa alamat lengkap, misalnya D:\\SmartFoot_Data.", e_BAD_ROUNDS: "Jumlah putaran harus 1 atau 3.",
    e_TIMEOUT: "Scanner tidak merespons. Cabut dan colok kembali kabel USB, lalu ulangi.",
    accept: "Terima", acceptAnyway: "Tetap terima", retake: "Ulangi", saving: "Menyimpan…",
    done: "Sesi selesai", doneSub: "Semua pemindaian tersimpan.", saved: "Tersimpan di", scans: "pemindaian", newS: "Sesi baru",
    set: "Pengaturan", setSub: "Pengaturan: folder data, scanner, salinan cadangan, dan kalibrasi skala.",
    teamOnly: "Area tim · tidak ditampilkan di alur perawat",
    rnd: "Jumlah putaran bawaan", r1: "1 putaran", r3: "3 putaran", rndNote: "Tercatat di setiap file sesi.",
    dataf: "Folder data", save: "Simpan", saved2: "Tersimpan ✓", station: "Kode laptop", stationNote: "Awalan ID pasien, unik per laptop (1–4 huruf/angka). Jangan sama dengan laptop lain.",
    scanner: "Scanner", auto: "Otomatis (LiDE 300)", refresh: "↻ Refresh",
    cal: "Kalibrasi skala", calNote: "Dikalibrasi 13 Nov 2026 dengan penggaris baja dan plastik di atas akrilik (kasing akhir). Memanjang (arah gerak scanner): 300,7 dpi = 11,84 piksel per mm. Melintang: 299,5 dpi = 11,79 piksel per mm. Nilai ini dicatat di setiap file sesi.",
    calCheck: "Cek ulang setelah scanner dipindah, spacer diganti, atau akrilik diganti: pindai penggaris baja di bawah kain putih, lalu beri tahu tim jika jarak 1 mm tidak lagi 11,84 piksel.",
    back: "← Kembali", cancel: "Batalkan sesi", cancelSure: "Yakin? Tekan lagi untuk batal",
    e_NOT_FOUND: "Scanner tidak ditemukan. Periksa kabel USB dan pastikan scanner menyala.",
    e_LOCKED: "Scan tidak bisa dimulai. Pastikan kunci transportasi scanner (LOCK) terbuka dan scanner terpasang lengkap (kaca bawaan scanner harus terpasang), lalu ulangi.",
    e_BUSY: "Scanner sedang sibuk. Tunggu beberapa detik lalu coba lagi.",
    e_DISK_FULL: "Disk penuh. Kosongkan ruang (satu pemindaian butuh sekitar 35 MB) atau ganti folder data di Settings.",
    savedBtn: "✓ Tersimpan", browse: "Pilih folder…", bkDone: "Perhatian: salinan kedua belum berhasil. Data hanya ada di satu disk sampai folder salinan bisa diakses lagi.",
    e_BAD_STATION: "Kode laptop harus 1–4 huruf atau angka.", e_SCAN_FAILED: "Pemindaian gagal", e_ID_USED: "ID ini sudah dipakai. Gunakan ID lain.", e_EMPTY_ID: "ID pasien kosong.", e_OP: "Isi inisial operator.",
    retry: "Coba lagi",
    resBtn: "Detail pengukuran →", res: "Detail pengukuran", resSub: "Semua angka dari pemindaian. Tanpa label datar/normal/tinggi.",
    adjust: "Sesuaikan titik acuan", adjustOff: "Selesai menyesuaikan",
    resNone: "Belum ada sesi yang selesai.", resSess: "Sesi", resPts: "Seret dua titik putih: pusat tumit (bawah) dan ujung jari ke-2 (atas), lalu lepas. Angka dihitung ulang.",
    resCorr: "Titik dikoreksi", yes: "ya", no: "tidak", resAuto: "Kembalikan otomatis",
    resUncal: "Belum dikalibrasi: angka memakai 300 dpi nominal, jangan dipakai sebagai hasil akhir.", resNoMeas: "Pemindaian ini tidak bisa diukur (kaki tidak ditemukan).",
    resNoLabel: "Tidak ada label klasifikasi. Batas AI belum divalidasi untuk posisi duduk.",
    resGap: "Dasar jari dari celah antar jari", resProp: "Dasar jari diperkirakan 75% panjang (jari menempel)", resCsv: "File CSV",
    scChecking: "Memeriksa scanner…", scReady: "Scanner siap", scMissing: "Scanner tidak terdeteksi", scAttn: "Scanner perlu dicek",
    scMissingMsg: "Cabut lalu colok kembali kabel USB scanner, dan pastikan scanner menyala.",
    scNullMsg: "Windows memasang driver kosong (Null Driver) sehingga scanner tidak bisa dipakai. Cabut lalu colok kembali kabel USB; jika tetap begini, hubungi tim.",
    scPanel: "Pemeriksaan scanner", scState: "Status", scFound: "Perangkat terdeteksi", scNone: "tidak ada", scRoute: "Jalur yang dipakai", scNullRow: "Null Driver",
    scDriverless: "Driverless (disarankan)", scCanon: "Driver Canon", scOther: "Lainnya", scNo: "tidak ada", scYes: "ADA, cabut dan colok kembali",
    scTest: "Tes scanner", scTesting: "Menguji… (sampai ±25 detik)", scTestNote: "Memindai strip kecil tanpa kaki di kaca. Membuktikan scanner benar-benar bisa memindai dan kunci LOCK terbuka.",
    scLast: "Tes terakhir", scNever: "belum ada", scPass: "berhasil", scFail: "gagal",
    unf: "Sesi belum selesai", unfSub: "Aplikasi tertutup di tengah sesi ini. Lanjutkan dari pemindaian berikutnya.", unfOf: "dari", unfScans: "pemindaian tersimpan", unfBy: "operator", resume: "Lanjutkan sesi", resuming: "Membuka…",
    e_NOT_RESUMABLE: "Sesi ini sudah selesai atau dibatalkan.",
    bkLabel: "Salinan data kedua", bkNote: "Folder di disk, USB atau cloud lain. Setiap pemindaian yang diterima disalin otomatis. Kosongkan untuk mematikan.",
    bkNow: "Cadangkan sekarang", bkNowBusy: "Menyalin…", bkOff: "Salinan kedua belum diatur (data hanya ada di satu disk).", bkOk: "Salinan terakhir berhasil", bkFiles: "file", bkNever: "Belum ada salinan sejak aplikasi dibuka.",
    bkGone: "Folder salinan tidak bisa diakses (USB terlepas?). Data baru belum tersalin.", bkFail: "Salinan terakhir gagal",
    e_BACKUP_SAME: "Folder salinan harus berbeda dari folder data (bukan di dalamnya, bukan induknya).", e_BACKUP_FAILED: "Folder salinan tidak bisa dipakai", e_BACKUP_OFF: "Folder salinan belum diatur.",
    ddSaved: "Data disimpan di", ddChange: "Ubah", ddBad: "Folder data tidak bisa diakses",
    ddBadHint: "Colokkan kembali flash disk atau drive-nya, atau pilih folder lain lewat Ubah. Pemindaian belum bisa dimulai.",
    setupT: "Selamat datang di SmartFoot", setupSub: "Sebelum mulai, tentukan di mana hasil pemindaian disimpan di laptop ini.",
    setupData: "Folder data", required: "wajib", setupDataNote: "Semua sesi dan pemindaian disimpan di sini, di disk laptop atau di flash disk.",
    setupBk: "Salinan kedua (opsional)", setupBkNote: "Folder di disk lain, USB, atau cloud. Kosongkan untuk melewati; data lalu hanya ada di satu disk.",
    setupCode: "Kode laptop", setupGo: "Simpan dan mulai →", setupSaving: "Menyimpan…",
    kFL: "Panjang kaki", kFW: "Lebar kaki", kArea: "Luas kontak", kToeless: "Luas tanpa jari", kA: "Tumit (A)", kB: "Lengkung (B)", kC: "Depan (C)", kAI: "Arch index (AI)", kCSI: "Chippaux-Smirak (%)",
  },
  en: {
    steps: ["Start", "Scan", "Review", "Done"], team: "⚙ Settings",
    start: "New session", startSub: "Enter brief patient details. No name needed.",
    pid: "Patient ID", age: "Age", sex: "Sex", f: "Female", m: "Male",
    op: "Operator initials", opHint: "required", note: "Note (optional)", go: "Start scanning", starting: "Checking the scanner…",
    scan: "Scan foot", scanSub: "Place the foot on the glass, heel against the stop. Keep still until the scan ends.",
    left: "Left foot", right: "Right foot", putaran: "Round", doScan: "Start scan", scanning: "Scanning… keep the foot still", ready: "Ready",
    review: "Review result", reviewSub: "Blue line = detected foot edge. Make sure it follows the foot.",
    chkFound: "Foot detected", chkInside: "Whole foot inside the area",
    wNoFoot: "No foot detected. Make sure the foot rests on the glass, then retake.",
    wEdge: "The foot touches the image edge. Move it away from the edge, then retake.",
    wOdd: "The shape does not look like a foot (too small, too large or too elongated). Make sure a foot was scanned.",
    wTwo: "There is a second object on the glass. Only one is outlined. Remove the other object, then retake.",
    e_BAD_IMAGE: "The scan is damaged or too small. Retake the scan.",
    dmg: "Some sessions are damaged or incomplete", dmgHint: "Do not delete them. Tell the team.",
    e_BAD_ID: "The patient ID must start with a letter or digit, be at most 32 characters and not be a reserved Windows name (CON, NUL, COM1...).",
    e_BAD_OP: "The operator initials must start with a letter or digit and be at most 8 characters.", e_EMPTY_OP: "Enter the operator initials.",
    e_BAD_AGE: "The age must be a whole number from 0 to 120.", e_BAD_SEX: "Choose the sex.", e_BAD_DIR: "The folder must be a full path, for example D:\\SmartFoot_Data.", e_BAD_ROUNDS: "Rounds must be 1 or 3.",
    e_TIMEOUT: "The scanner is not answering. Unplug and replug the USB cable, then try again.",
    accept: "Accept", acceptAnyway: "Accept anyway", retake: "Retake", saving: "Saving…",
    done: "Session complete", doneSub: "All scans are saved.", saved: "Saved to", scans: "scans", newS: "New session",
    set: "Settings", setSub: "Settings: data folder, scanner, backup copy and scale calibration.",
    teamOnly: "Team area · not shown in the nurse flow",
    rnd: "Default number of rounds", r1: "1 round", r3: "3 rounds", rndNote: "Recorded in every session file.",
    dataf: "Data folder", save: "Save", saved2: "Saved ✓", station: "Laptop code", stationNote: "Prefix of patient IDs, unique per laptop (1–4 letters/digits). Never the same as another laptop.",
    scanner: "Scanner", auto: "Automatic (LiDE 300)", refresh: "↻ Refresh",
    cal: "Scale calibration", calNote: "Calibrated on 13 Nov 2026 with a steel and a plastic ruler on the acrylic (final case). Lengthwise (scanner travel): 300.7 dpi = 11.84 pixels per mm. Across: 299.5 dpi = 11.79 pixels per mm. These values are recorded in every session file.",
    calCheck: "Check again after the scanner is moved, the spacer is changed or the acrylic is replaced: scan the steel ruler under the white cloth and tell the team if 1 mm is no longer 11.84 pixels.",
    back: "← Back", cancel: "Cancel session", cancelSure: "Sure? Press again to cancel",
    e_NOT_FOUND: "Scanner not found. Check the USB cable and that the scanner is switched on.",
    e_LOCKED: "The scan could not start. Check that the scanner's transport LOCK switch is open and that the scanner is fully assembled (its own glass panel must be in place), then try again.",
    e_BUSY: "The scanner is busy. Wait a few seconds and try again.",
    e_DISK_FULL: "The disk is full. Free some space (one scan needs about 35 MB) or change the data folder in Settings.",
    savedBtn: "✓ Saved", browse: "Choose folder…", bkDone: "Warning: the second copy did not succeed. The data exists on one disk only until the backup folder is reachable again.",
    e_BAD_STATION: "The laptop code needs 1–4 letters or digits.", e_SCAN_FAILED: "Scan failed", e_ID_USED: "This ID is already used. Pick another ID.", e_EMPTY_ID: "The patient ID is empty.", e_OP: "Enter the operator initials.",
    retry: "Try again",
    resBtn: "Measurement details →", res: "Measurement details", resSub: "All numbers from the scans. No flat/normal/high label.",
    adjust: "Adjust reference points", adjustOff: "Done adjusting",
    resNone: "No finished session yet.", resSess: "Session", resPts: "Drag the two white points: heel centre (bottom) and 2nd-toe tip (top), then release. Numbers are recalculated.",
    resCorr: "Points corrected", yes: "yes", no: "no", resAuto: "Reset to automatic",
    resUncal: "Not calibrated: numbers use nominal 300 dpi, do not use them as final results.", resNoMeas: "This scan could not be measured (no foot found).",
    resNoLabel: "No classification label. AI cutoffs are not validated for seated posture.",
    resGap: "Toe base from the gaps between toes", resProp: "Toe base estimated at 75% of length (toes touching)", resCsv: "CSV file",
    scChecking: "Checking scanner…", scReady: "Scanner ready", scMissing: "Scanner not detected", scAttn: "Scanner needs attention",
    scMissingMsg: "Unplug and replug the scanner's USB cable, and make sure the scanner is switched on.",
    scNullMsg: "Windows installed an empty driver (Null Driver), so the scanner cannot be used. Unplug and replug the USB cable; if it stays like this, call the team.",
    scPanel: "Scanner check", scState: "Status", scFound: "Devices detected", scNone: "none", scRoute: "Route in use", scNullRow: "Null Driver",
    scDriverless: "Driverless (preferred)", scCanon: "Canon driver", scOther: "Other", scNo: "none", scYes: "PRESENT, unplug and replug",
    scTest: "Test scanner", scTesting: "Testing… (up to ~25 s)", scTestNote: "Scans a small strip with no foot on the glass. Proves the scanner can really scan and the LOCK switch is open.",
    scLast: "Last test", scNever: "none yet", scPass: "passed", scFail: "failed",
    unf: "Unfinished session", unfSub: "The app closed in the middle of this session. Continue from the next scan.", unfOf: "of", unfScans: "scans saved", unfBy: "operator", resume: "Resume session", resuming: "Opening…",
    e_NOT_RESUMABLE: "This session is already finished or cancelled.",
    bkLabel: "Second data copy", bkNote: "A folder on another disk, USB drive or cloud folder. Every accepted scan is copied automatically. Leave empty to turn it off.",
    bkNow: "Back up now", bkNowBusy: "Copying…", bkOff: "No second copy set (the data lives on one disk only).", bkOk: "Last copy succeeded", bkFiles: "files", bkNever: "No copy yet since the app opened.",
    bkGone: "The backup folder cannot be reached (USB unplugged?). New data has not been copied.", bkFail: "The last copy failed",
    e_BACKUP_SAME: "The backup folder must be different from the data folder (not inside it, not its parent).", e_BACKUP_FAILED: "The backup folder cannot be used", e_BACKUP_OFF: "No backup folder is set.",
    ddSaved: "Data saved to", ddChange: "Change", ddBad: "The data folder cannot be reached",
    ddBadHint: "Plug the flash disk or drive back in, or choose another folder with Change. Scanning cannot start yet.",
    setupT: "Welcome to SmartFoot", setupSub: "Before you start, choose where the scans are saved on this laptop.",
    setupData: "Data folder", required: "required", setupDataNote: "Every session and scan is saved here, on the laptop disk or on a flash disk.",
    setupBk: "Second copy (optional)", setupBkNote: "A folder on another disk, USB drive or cloud. Leave empty to skip; the data then lives on one disk only.",
    setupCode: "Laptop code", setupGo: "Save and start →", setupSaving: "Saving…",
    kFL: "Foot length", kFW: "Foot width", kArea: "Contact area", kToeless: "Area without toes", kA: "Hindfoot (A)", kB: "Midfoot (B)", kC: "Forefoot (C)", kAI: "Arch index (AI)", kCSI: "Chippaux-Smirak (%)",
  },
};

let L = localStorage.getItem("lang") || "id";
const t = (k) => T[L][k];
const $ = (s) => document.querySelector(s);
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

// ---- scanner status: chip in the top bar, gate on the start screen, panel in team settings ----
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

const st = { screen: "start", prev: "start", form: { pid: "", op: "", age: "", sex: "f", note: "" }, sess: null, idx: 0, scan: null, error: "", busy: false, settings: null, draft: {}, flash: null, ferr: null, scanners: [], unfinished: undefined, damaged: [] };
// text fields in Settings with their own Save button: current value and "unchanged" test
const FIELDS = {
  dir: { get: (c) => c.data_dir, same: (v, c) => v.trim() === c.data_dir },
  station: { get: (c) => c.station, same: (v, c) => v.trim().toUpperCase() === c.station },
  bk: { get: (c) => c.backup_dir ?? "", same: (v, c) => v.trim() === (c.backup_dir ?? "") },
};
const STEP = { start: 0, scan: 1, review: 2, done: 3 };

function errText(raw) {
  if (/not enough space|disk is full|os error 112/i.test(String(raw))) return esc(t("e_DISK_FULL"));
  const m = String(raw).match(/^([A-Z_]+):\s*(.*)$/s);
  if (!m) return esc(raw);
  const known = T[L]["e_" + m[1]];
  return esc(known ? (m[1] === "SCAN_FAILED" || m[1] === "BACKUP_FAILED" ? `${known}: ${m[2]}` : known) : raw);
}
const errBox = () => (st.error ? `<div class="err">${errText(st.error)}</div>` : "");

function steps() {
  const p = STEP[st.screen] ?? -1;
  $("#steps").innerHTML = t("steps").map((n, i) => `<div class="step ${i < p ? "done" : i == p ? "on" : ""}"><i>${i < p ? "✓" : i + 1}</i><b>${n}</b></div>`).join("");
}

const seqOf = (rounds) => { const s = []; for (let r = 1; r <= rounds; r++) { s.push(["right", r]); s.push(["left", r]); } return s; };
const roundsBar = () => `<div class="rounds">${st.sess.seq.map((s, i) => `<span class="${i < st.idx ? "d" : i == st.idx ? "c" : ""}">${s[0] == "left" ? "L" : "R"}${s[1]}</span>`).join("")}</div>`;

const OUT = "M112 258c-16 8-38 6-44-12-6-20 4-44 2-70-2-26-20-52-12-86 7-30 34-44 56-36 24 8 30 40 26 70-3 24 6 46 8 70 2 22-8 56-36 64z";
const guideFoot = (side) => `<svg viewBox="0 0 210 297"><g transform="${side == "left" ? "translate(210,0) scale(-1,1)" : ""}" opacity=".55"><path d="${OUT}" fill="#d7b79e"/><g fill="#d7b79e"><ellipse cx="114" cy="62" rx="13" ry="16"/><ellipse cx="135" cy="70" rx="8" ry="11"/><ellipse cx="148" cy="82" rx="7" ry="10"/><ellipse cx="156" cy="97" rx="6" ry="9"/><ellipse cx="160" cy="112" rx="5" ry="8"/></g></g></svg>`;

const cancelBtn = () => `<div class="row" style="margin-top:22px"><button class="btn bad" id="cancel" style="padding:10px 18px;font-size:15px">${t("cancel")}</button></div>`;
function wireCancel() {
  const b = $("#cancel"); if (!b) return; let armed = false;
  b.onclick = async () => {
    if (!armed) { armed = true; b.textContent = t("cancelSure"); b.classList.add("armed"); setTimeout(() => { armed = false; if (b.isConnected) { b.textContent = t("cancel"); b.classList.remove("armed"); } }, 4000); return; }
    try { await invoke("cancel_session", { pid: st.sess.pid }); } catch (e) { st.error = e; render(); return; }
    st.form = { pid: "", op: st.form.op, age: "", sex: "f", note: "" }; st.sess = null; st.scan = null; st.error = ""; st.screen = "start"; render();
  };
}

// ---- measurement details ("Detail pengukuran"): all values, reference points can be adjusted ----
const rs = { sessions: [], pid: null, sess: null, cur: null, view: null, busy: false, adjust: false, back: "settings" };
async function loadView() { rs.view = rs.cur ? await invoke("get_scan_view", { pid: rs.pid, side: rs.cur.side, round: rs.cur.round }) : null; }
async function loadSession() {
  rs.sess = rs.pid ? await invoke("get_session", { pid: rs.pid }) : null;
  const f = rs.sess?.scans[0];
  rs.cur = f ? { side: f.side, round: f.round } : null;
  await loadView();
}
async function openResearch(pid, back = "settings") {
  rs.back = back; rs.adjust = false;
  if (pid) rs.pid = pid;
  try {
    rs.sessions = (await invoke("list_sessions")).filter((x) => x.scans > 0);
    if (!rs.sessions.find((x) => x.pid === rs.pid)) rs.pid = rs.sessions[0]?.pid ?? null;
    await loadSession(); st.error = "";
  } catch (e) { st.error = e; }
  st.screen = "research"; render();
}
const f1 = (x, d = 1) => (x == null ? "–" : Number(x).toFixed(d));
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
    // the two reference points are only shown (and draggable) after "Sesuaikan titik acuan"
    if (rs.adjust) g += `<circle id="ph" cx="${H[0]}" cy="${H[1]}" r="11" fill="#fff" stroke="#1f5fbf" stroke-width="4" style="cursor:grab"/><circle id="pj" cx="${J[0]}" cy="${J[1]}" r="11" fill="#fff" stroke="#1f5fbf" stroke-width="4" style="cursor:grab"/>`;
  }
  const svg = `<svg id="rsvg" viewBox="0 0 ${v.preview_w} ${v.preview_h}" style="touch-action:none;width:100%;height:100%"><image href="${v.preview}" width="${v.preview_w}" height="${v.preview_h}"/>${g}</svg>`;
  const kv = (k) => { const [val, d, u] = VAL[k](m); return `<div class="kv"><span style="display:flex;align-items:center">${esc(X[L]["i_" + k][0])}${ib(k)}</span><b>${num(val, d)}${u ? " " + u : ""} <em class="mp2">${precText(k)}</em></b></div>`; };
  const nums = !m ? `<div class="warn">${t("resNoMeas")}</div>` :
    `${["fl", "fw", "area", "toeless", "A", "B", "C", "ai", "csi"].map(kv).join("")}
     <p class="muted" style="margin:12px 0 4px">${m.toe_base_method === "gap" ? t("resGap") : t("resProp")}</p>
     <span class="chip ${m.corrected ? "warn" : "ok"}">${t("resCorr")}: ${m.corrected ? t("yes") : t("no")}</span>
     ${m.scale.calibrated ? "" : `<div class="warn" style="margin-top:14px">${t("resUncal")}</div>`}`;
  return { svg, nums };
}
function wireResearchDrag() {
  const svg = $("#rsvg"); if (!svg || !$("#ph")) return;
  const toSvg = (e) => { const q = svg.createSVGPoint(); q.x = e.clientX; q.y = e.clientY; return q.matrixTransform(svg.getScreenCTM().inverse()); };
  let drag = null;
  ["ph", "pj"].forEach((id) => {
    const c = $("#" + id);
    c.onpointerdown = (e) => { drag = c; c.setPointerCapture(e.pointerId); };
    c.onpointermove = (e) => { if (drag !== c) return; const q = toSvg(e); c.setAttribute("cx", q.x); c.setAttribute("cy", q.y); };
    c.onpointerup = async () => {
      if (drag !== c) return; drag = null;
      const d = rs.view.down, at = (i, k) => Number($(i).getAttribute(k)) * d;
      await remeasure([at("#ph", "cx"), at("#ph", "cy")], [at("#pj", "cx"), at("#pj", "cy")]);
    };
  });
}
async function remeasure(heel, toe) {
  rs.busy = true; $("#rsnums").style.opacity = ".4";
  try { rs.view.measurements = await invoke("remeasure", { pid: rs.pid, side: rs.cur.side, round: rs.cur.round, heel, toe }); st.error = ""; }
  catch (e) { st.error = e; }
  rs.busy = false; paintResearch();
}
function paintResearch() {
  const o = overlayAndNumbers();
  $("#rsimg").innerHTML = o.svg; $("#rsnums").innerHTML = o.nums; $("#rsnums").style.opacity = "1";
  const v = rs.view;
  if (v?.measurements) wireInfo($("#rsnums"), { img: v.preview, w: v.preview_w, h: v.preview_h, down: v.down, m: v.measurements });
  wireResearchDrag();
}
function researchScreen(m) {
  const back = () => { st.screen = rs.back; rs.adjust = false; render(); };
  if (!rs.sessions.length) {
    m.innerHTML = `<h1>${t("res")}</h1>${errBox()}<div class="card"><p class="sub" style="margin:0">${t("resNone")}</p></div><div class="row" style="margin-top:22px"><button class="btn ghost" id="rback">${t("back")}</button></div>`;
    $("#rback").onclick = back; return;
  }
  m.innerHTML = `<h1>${t("res")}</h1><p class="sub">${t("resSub")}</p>${errBox()}
    <div class="row" style="margin-bottom:16px"><label style="margin:0">${t("resSess")}</label><select id="rs_sess" style="width:auto">${rs.sessions.map((x) => `<option ${x.pid == rs.pid ? "selected" : ""}>${esc(x.pid)}</option>`).join("")}</select>
      <div class="seg" style="flex:none">${(rs.sess?.scans ?? []).map((x) => `<button data-sc="${x.side}|${x.round}" class="${rs.cur && rs.cur.side == x.side && rs.cur.round == x.round ? "on" : ""}" style="padding:10px 16px">${x.side == "left" ? "L" : "R"}${x.round}</button>`).join("")}</div></div>
    <div class="grid2"><div class="card"><div class="scanwin" id="rsimg" style="background:#10151f"></div>
      ${rs.adjust ? `<p class="muted" style="margin:12px 0 0">${t("resPts")}</p>` : ""}
      <div class="row" style="margin-top:12px"><button class="btn ${rs.adjust ? "ok" : "ghost"}" id="rs_adj" style="padding:10px 16px;font-size:15px">${rs.adjust ? "✓ " + t("adjustOff") : t("adjust")}</button>
        ${rs.adjust ? `<button class="btn ghost" id="rs_auto" style="padding:10px 16px;font-size:15px">${t("resAuto")}</button>` : ""}</div></div>
    <div class="card" id="rsnums"></div></div>
    <p class="muted" style="margin-top:14px">${t("resNoLabel")}</p><p class="muted">${t("resCsv")}: ${esc(rs.view?.csv ?? "")}</p>
    <div class="row" style="margin-top:12px"><button class="btn ghost" id="rback">${t("back")}</button></div>`;
  paintResearch();
  $("#rs_sess").onchange = async (e) => { rs.pid = e.target.value; try { await loadSession(); } catch (x) { st.error = x; } render(); };
  m.querySelectorAll("[data-sc]").forEach((b) => (b.onclick = async () => { const [side, round] = b.dataset.sc.split("|"); rs.cur = { side, round: Number(round) }; try { await loadView(); } catch (x) { st.error = x; } render(); }));
  $("#rs_adj").onclick = () => { rs.adjust = !rs.adjust; render(); };
  if ($("#rs_auto")) $("#rs_auto").onclick = () => remeasure(null, null);
  $("#rback").onclick = back;
}

async function render() {
  steps();
  const m = $("#main");
  $("#teamBtn").textContent = t("team");
  const s = st.screen;

  if (s === "start") {
    if (!st.form.pid) { try { st.form.pid = await invoke("new_session_id"); } catch (e) { st.error = e; } }
    if (st.unfinished === undefined) {
      try { st.unfinished = await invoke("find_unfinished"); } catch { st.unfinished = null; }
      try { st.damaged = await invoke("damaged_sessions"); } catch { st.damaged = []; }
    }
    const dmg = st.damaged?.length ? `<div class="warn">${t("dmg")}: <b>${st.damaged.map(esc).join(", ")}</b>. ${t("dmgHint")}</div>` : "";
    const f = st.form, u = st.unfinished;
    const banner = u ? `<div class="card" style="margin-bottom:20px;border:2px solid var(--warn)"><div class="row"><div><div style="font-size:20px;font-weight:800">${t("unf")}</div>
        <div class="muted" style="margin:4px 0 8px">${t("unfSub")}</div><div><b>${esc(u.pid)}</b> · ${u.done} ${t("unfOf")} ${u.total} ${t("unfScans")} · ${t("unfBy")} ${esc(u.operator)}</div></div>
        <div class="row right"><button class="btn bad" id="unf_cancel" style="padding:14px 22px;font-size:17px">${t("cancel")}</button><button class="btn" id="unf_go" style="padding:14px 26px;font-size:18px">${t("resume")} →</button></div></div></div>` : "";
    m.innerHTML = `<h1>${t("start")}</h1><p class="sub">${t("startSub")}</p><div id="ddline"></div><div id="scgate"></div>${dmg}${banner}${errBox()}<div class="card"><div class="grid2">
      <div class="field"><label>${t("pid")}</label><input id="f_pid" value="${esc(f.pid)}"></div>
      <div class="field"><label>${t("op")} <span class="muted">(${t("opHint")})</span></label><input id="f_op" value="${esc(f.op)}" placeholder="RF" maxlength="4"></div>
      <div class="field"><label>${t("age")}</label><input id="f_age" type="number" min="0" max="120" value="${esc(f.age)}" placeholder="45"></div>
      <div class="field"><label>${t("sex")}</label><div class="seg"><button class="${f.sex == "f" ? "on" : ""}" data-sex="f">${t("f")}</button><button class="${f.sex == "m" ? "on" : ""}" data-sex="m">${t("m")}</button></div></div>
      <div class="field" style="grid-column:1/-1"><label>${t("note")}</label><input id="f_note" value="${esc(f.note)}"></div></div>
      <div class="row"><button class="btn right" id="go" ${st.busy ? "disabled" : ""}>${st.busy ? t("starting") : t("go") + " →"}</button></div></div>`;
    const keep = () => { f.pid = $("#f_pid").value; f.op = $("#f_op").value; f.age = $("#f_age").value; f.note = $("#f_note").value; };
    m.querySelectorAll("[data-sex]").forEach((b) => (b.onclick = () => { keep(); f.sex = b.dataset.sex; render(); }));
    paintGate();
    invoke("data_dir_status").then((r) => { dd = r; paintGate(); }).catch(() => {});
    if (u) {
      $("#unf_go").onclick = async () => {
        $("#unf_go").disabled = true; $("#unf_go").textContent = t("resuming");
        try {
          const r = await invoke("resume_session", { pid: u.pid });
          st.sess = { ...r.start, seq: seqOf(r.start.rounds) };
          st.idx = Math.min(r.idx, st.sess.seq.length - 1); st.error = ""; st.unfinished = null;
          if (r.pending) { st.scan = r.pending; st.screen = "review"; }
          else if (r.idx >= st.sess.seq.length) { await invoke("finish_session", { pid: u.pid }); st.scan = null; st.screen = "done"; }
          else { st.scan = null; st.screen = "scan"; }
        } catch (e) { st.error = e; st.unfinished = undefined; }
        render();
      };
      let armed = false; const cb = $("#unf_cancel");
      cb.onclick = async () => {
        if (!armed) { armed = true; cb.textContent = t("cancelSure"); cb.classList.add("armed"); setTimeout(() => { armed = false; if (cb.isConnected) { cb.textContent = t("cancel"); cb.classList.remove("armed"); } }, 4000); return; }
        try { await invoke("cancel_session", { pid: u.pid }); } catch (e) { st.error = e; }
        st.unfinished = undefined; st.form.pid = ""; render();
      };
    }
    $("#go").onclick = async () => {
      keep(); st.error = "";
      if (!f.op.trim()) { m.querySelector("h1").insertAdjacentHTML("afterend", `<div class="err">${t("e_OP")}</div>`); return; }
      const ageNum = f.age === "" ? null : Number(f.age);
      if (ageNum !== null && (!Number.isInteger(ageNum) || ageNum < 0 || ageNum > 120)) { m.querySelector("h1").insertAdjacentHTML("afterend", `<div class="err">${t("e_BAD_AGE")}</div>`); return; }
      st.busy = true; render();
      try {
        const info = await invoke("start_session", { pid: f.pid, operator: f.op, age: ageNum, sex: f.sex, note: f.note });
        st.sess = { ...info, seq: seqOf(info.rounds) };
        st.idx = 0; st.scan = null; st.screen = "scan"; st.unfinished = undefined;
        sc.lock = false; if (sc.test && !sc.test.ok) sc.test = null;
      } catch (e) { st.error = e; noteScanError(e); }
      st.busy = false; render(); pollScanner();
    };
  }

  if (s === "setup") {
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
        st.error = ""; st.setup = null; st.screen = "start"; st.form.pid = ""; st.unfinished = undefined;
        await render(); pollScanner(); startTour(false);
      } catch (e) { st.error = e; render(); }
    };
  }

  if (s === "scan") {
    const [side, r] = st.sess.seq[st.idx];
    m.innerHTML = `<h1>${t("scan")} · ${t(side)}</h1><p class="sub">${t("scanSub")}</p><div class="grid2"><div class="card"><div class="scanwin">${guideFoot(side)}</div></div>
      <div class="card"><span class="chip">${t("putaran")} ${r}/${st.sess.rounds}</span>${roundsBar()}${errBox()}<div id="sstate" style="margin:20px 0 14px;color:var(--mute)">${t("ready")}</div><div class="bar"><b id="sbar"></b></div>
      <div class="row" style="margin-top:26px"><button class="btn" id="sgo">${st.error ? t("retry") : t("doScan")}</button></div>${cancelBtn()}</div></div>`;
    wireCancel();
    $("#sgo").onclick = async () => {
      st.error = ""; $("#sgo").disabled = true; $("#sstate").textContent = t("scanning");
      let p = 0; const iv = setInterval(() => { p += (95 - p) * 0.04; $("#sbar").style.width = p + "%"; }, 400);
      try {
        st.scanning = true;
        st.scan = await invoke("scan_foot", { pid: st.sess.pid, side, round: r });
        clearInterval(iv); st.screen = "review"; sc.lock = false; if (sc.test && !sc.test.ok) sc.test = null;
      } catch (e) { clearInterval(iv); st.error = e; noteScanError(e); }
      st.scanning = false;
      render();
    };
  }

  if (s === "review") {
    const [side, r] = st.sess.seq[st.idx], sc = st.scan, w = sc.warnings;
    const noFoot = w.includes("NO_FOOT"), edge = w.includes("EDGE"), ok = !w.length;
    const warnText = [noFoot ? t("wNoFoot") : "", edge ? t("wEdge") : "", w.includes("ODD_SHAPE") ? t("wOdd") : "", w.includes("TWO_OBJECTS") ? t("wTwo") : ""].filter(Boolean).join(" ");
    // outline switch: ON every time a new scan is reviewed. Older pending scans (resume) may lack `plain`.
    const canTog = !noFoot && sc.plain;
    m.innerHTML = `<h1>${t("review")} · ${t(side)} ${r}/${st.sess.rounds}</h1><p class="sub">${t("reviewSub")}</p>${errBox()}<div class="grid2"><div class="card"><div class="scanwin" style="background:#10151f"><img id="rvimg" src="${sc.preview}" alt="">
        ${canTog ? `<label class="tog"><input type="checkbox" id="rvtog" checked> ${X[L].outline}</label>` : ""}</div></div>
      <div class="card" id="rvcard"><ul class="checks"><li><span class="dot ${noFoot ? "bad" : ""}"></span>${t("chkFound")}</li><li><span class="dot ${edge || noFoot ? "bad" : ""}"></span>${t("chkInside")}</li></ul>
      ${ok ? "" : `<div class="warn">${warnText}</div>`}
      <div class="row" style="margin-top:24px"><button class="btn ${ok ? "bad" : ""}" id="retake">↺ ${t("retake")}</button>
      <button class="btn ${ok ? "ok" : "ghost"} right" id="accept">${ok ? "✓ " + t("accept") : t("acceptAnyway")}</button></div>
      ${noFoot ? "" : metricsBlock(sc.measurements)}${cancelBtn()}</div></div>`;
    wireCancel();
    if ($("#rvtog")) $("#rvtog").onchange = (e) => { $("#rvimg").src = e.target.checked ? sc.preview : sc.plain; };
    if (sc.measurements) wireInfo($("#rvcard"), { img: sc.plain || sc.preview, w: sc.preview_w, h: sc.preview_h, down: sc.width_px / sc.preview_w, m: sc.measurements });
    $("#retake").onclick = async () => {
      try { await invoke("retake_scan", { pid: st.sess.pid, side, round: r }); st.scan = null; st.screen = "scan"; st.error = ""; } catch (e) { st.error = e; }
      render();
    };
    $("#accept").onclick = async () => {
      $("#accept").disabled = true; $("#accept").textContent = t("saving");
      try {
        await invoke("accept_scan", { pid: st.sess.pid, side, round: r, seconds: sc.seconds, widthPx: sc.width_px, heightPx: sc.height_px, warnings: w });
        if (st.idx < st.sess.seq.length - 1) { st.idx++; st.screen = "scan"; }
        else { await invoke("finish_session", { pid: st.sess.pid }); st.screen = "done"; try { bk = await invoke("backup_status"); } catch { bk = null; } }
        st.scan = null; st.error = "";
      } catch (e) { st.error = e; }
      render();
    };
  }

  if (s === "done") {
    const bkBad = bk?.enabled && (!bk.reachable || bk.last_err) ? `<div class="warn">${t("bkDone")}<br>${bkLine()}</div>` : "";
    m.innerHTML = `<h1>${t("done")}</h1><p class="sub">${t("doneSub")}</p>${bkBad}<div class="card"><div class="row" style="gap:30px"><div class="big" style="color:var(--ok)">✓</div><div><div style="font-size:22px;font-weight:700">${esc(st.sess.pid)} · ${st.sess.seq.length} ${t("scans")}</div><div class="sub" style="margin:4px 0 0">${t("saved")} ${esc(st.sess.dir)}</div></div></div>
      <div class="row" style="margin-top:26px"><button class="btn ghost" id="det">${t("resBtn")}</button><button class="btn right" id="new">${t("newS")}</button></div></div>`;
    $("#det").onclick = () => openResearch(st.sess.pid, "done");
    $("#new").onclick = () => { st.form = { pid: "", op: st.form.op, age: "", sex: "f", note: "" }; st.sess = null; st.screen = "start"; render(); };
  }

  if (s === "research") researchScreen(m);

  if (s === "settings") {
    const c = st.settings;
    const val = (f) => st.draft[f] ?? FIELDS[f].get(c);
    const saveBtn = (f, id) => st.flash === f
      ? `<button class="btn ok" id="${id}" style="padding:12px 18px;font-size:16px">${t("savedBtn")}</button>`
      : `<button class="btn ghost" id="${id}" style="padding:12px 18px;font-size:16px" ${FIELDS[f].same(val(f), c) ? "disabled" : ""}>${t("save")}</button>`;
    const browseBtn = (id) => `<button class="btn ghost" id="${id}" style="padding:12px 18px;font-size:16px">${t("browse")}</button>`;
    const fieldErr = (f) => (st.ferr?.f === f ? `<div class="err" style="margin:8px 0 0">${errText(st.ferr.e)}</div>` : "");
    m.innerHTML = `<h1>${t("set")}</h1><p class="sub">${t("setSub")}</p>${errBox()}<div class="grid2">
      <div class="card"><label>${t("rnd")}</label><div class="seg"><button class="${c.rounds == 1 ? "on" : ""}" data-r="1">${t("r1")}</button><button class="${c.rounds == 3 ? "on" : ""}" data-r="3">${t("r3")}</button></div><p class="muted" style="margin:8px 0 22px">${t("rndNote")}</p>
        <label>${t("dataf")}</label><div class="row"><input id="s_dir" value="${esc(val("dir"))}" style="flex:1">${browseBtn("s_dir_b")}${saveBtn("dir", "s_save")}</div>${fieldErr("dir")}
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
    m.querySelectorAll("[data-r]").forEach((b) => (b.onclick = () => persist({ rounds: Number(b.dataset.r) })));
    $("#s_save").onclick = () => persist({ data_dir: $("#s_dir").value.trim() }, "dir");
    $("#s_stsave").onclick = () => { st.form.pid = ""; persist({ station: $("#s_station").value }, "station"); };
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
  italicize($("aside")); italicize(m); paintChip();
}

$("#lang").onclick = () => { L = L == "id" ? "en" : "id"; localStorage.setItem("lang", L); render(); };
$("aside .row").insertAdjacentHTML("afterbegin", `<button class="lang" id="teamBtn"></button>`);
$("#teamBtn").onclick = async () => {
  if (st.screen === "settings") { st.screen = st.prev; }
  else {
    if (st.busy) return;
    st.prev = st.screen; st.screen = "settings"; st.error = ""; st.draft = {}; st.ferr = null; st.flash = null;
    try { st.settings = await invoke("get_settings"); try { bk = await invoke("backup_status"); } catch { bk = null; } st.scanners = await invoke("list_scanners").catch(() => []); } catch (e) { st.error = e; }
  }
  render();
};
$("#schip").onclick = () => { sc.phase = "checking"; paintChip(); pollScanner(); };
// first-run setup ("where is the data saved?") comes before anything else on a new laptop
(async () => {
  try { st.settings = await invoke("get_settings"); if (!st.settings.setup_done) st.screen = "setup"; } catch (e) { st.error = e; }
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

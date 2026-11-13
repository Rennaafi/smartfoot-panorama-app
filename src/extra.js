// SmartFoot extras: measurement info pop-ups (ⓘ) with animations, the guide, the first-run tour,
// the opening animation and the logo. Loaded after app.js and uses its globals (t, L, $, esc).

// ---------- precision of each value (ONE place: update after a new repeat-scan session) ----------
// Half of the range over 8 repeat placements of one right foot (R01-R07, R09; black cloth, lights on,
// final case with spacer), calibrated scale, 13 Nov 2026. R08 (curled toes) and R10 (edge) excluded.
const PREC = {
  fl: [1.9, "mm"], fw: [1.7, "mm"], ai: [0.007, ""], area: [3.7, "cm²"], toeless: [10.1, "cm²"],
  A: [2.4, "cm²"], B: [3.8, "cm²"], C: [4.2, "cm²"], csi: [3.6, ""],
};
const PREC_SRC = {
  id: "dari 8 pindaian ulang satu kaki (2 pindaian dengan jari menekuk / kaki di tepi tidak dihitung), 13 Nov 2026",
  en: "from 8 repeat scans of one foot (2 scans with curled toes / the foot at the edge not counted), 13 Nov 2026",
};
const TAPE = { mean: 244.8, tape: 245 }; // same 8 scans vs a tape measurement of the same foot

const REFS = {
  cav: 'P. R. Cavanagh and M. M. Rodgers, "The arch index: A useful measure from footprints," <i>J. Biomech.</i>, vol. 20, no. 5, pp. 547–551, 1987, doi: 10.1016/0021-9290(87)90255-7.',
  pap: 'M. O. Papuga and J. R. Burke, "The reliability of the Associate Platinum digital foot scanner in measuring previously developed footprint characteristics: A technical note," <i>J. Manipulative Physiol. Ther.</i>, vol. 34, no. 2, pp. 114–118, Feb. 2011, doi: 10.1016/j.jmpt.2010.12.008.',
};

const X = {
  id: {
    guideBtn: "? Panduan", outline: "Garis tepi", more: "Detail lainnya", replay: "↻ Putar ulang", close: "Tutup",
    prec: "Ketelitian", basis: "Dasar", refs: "Referensi", scaleH: "Cara skala bekerja", noMeas: "Angka tidak tersedia untuk pemindaian ini.",
    detNote: "Titik acuan bisa disesuaikan di Detail pengukuran setelah sesi selesai.",
    rulerCap: "Hasil pindai penggaris baja di bawah kain putih (13 Nov 2026). Tanda oranye = tiap 11,84 piksel (1 mm), garis biru = 10 mm = 118 piksel. Tanda oranye jatuh tepat di garis penggaris.",
    calSteps: ["Penggaris di atas akrilik", "Tutup dengan kain putih", "Pindai, lalu hitung piksel"],
    scale: "Scanner mengubah gambar menjadi titik-titik (piksel). Kami mengukur penggaris baja di atas kaca: 1 mm = 11,84 piksel memanjang dan 11,79 piksel melintang. Aplikasi menghitung piksel kaki lalu mengubahnya ke mm. Dicek 13 Nov 2026.",
    tape: `Rata-rata 8 pindaian ${TAPE.mean.toString().replace(".", ",")} mm vs pita ukur ${TAPE.tape} mm (selisih ${(TAPE.tape - TAPE.mean).toFixed(1).replace(".", ",")} mm).`,
    i_fl: ["Panjang kaki", "Jarak dari ujung tumit ke ujung jari terpanjang, diukur sepanjang sumbu kaki (garis dari tengah tumit ke ujung jari ke-2)."],
    i_fw: ["Lebar kaki", "Bagian telapak yang paling lebar, diukur tegak lurus sumbu kaki. Biasanya di bagian bola kaki."],
    i_ai: ["Arch Index (AI)", "Membandingkan luas bagian tengah telapak (B) dengan luas seluruh telapak tanpa jari (A + B + C): AI = B ÷ (A + B + C) [1]. Semakin besar AI, semakin banyak bagian lengkung kaki yang menyentuh kaca.",
      "Aplikasi tidak memberi label datar / normal / tinggi. Batas di literatur dibuat untuk kaki berdiri [1], sedangkan pengukuran ini dalam posisi duduk."],
    i_area: ["Luas kontak", "Luas seluruh telapak yang menyentuh kaca, termasuk jari kaki."],
    i_toeless: ["Luas tanpa jari", "Luas telapak dari tumit sampai pangkal jari, tanpa jari kaki. Inilah yang dibagi tiga (A, B, C) untuk Arch Index [1]."],
    i_A: ["Tumit (A)", "Sepertiga belakang dari telapak tanpa jari, yaitu bagian tumit [1]."],
    i_B: ["Lengkung (B)", "Sepertiga tengah dari telapak tanpa jari, yaitu bagian lengkung kaki [1]."],
    i_C: ["Depan (C)", "Sepertiga depan dari telapak tanpa jari, yaitu bagian bola kaki [1]."],
    i_csi: ["Chippaux-Smirak (%)", "Lebar tersempit bagian tengah (lengkung) dibagi lebar terlebar bagian depan, dikali 100 [1]. Semakin besar, semakin lebar bagian lengkung yang menyentuh kaca."],
    tour: [
      ["#steps", "Alur kerja", "Setiap pasien melewati 4 langkah: Mulai, Pindai, Tinjau, Selesai. Langkah yang aktif berwarna putih."],
      ["#schip", "Status scanner", "Hijau = scanner siap. Kuning atau merah = ada masalah, ketuk untuk memeriksa lagi."],
      ["#main .card", "Data pasien", "Isi usia, jenis kelamin, dan inisial operator. Tidak perlu nama. ID dibuat otomatis."],
      ["#guideBtn", "Panduan", "Langkah lengkap dengan gambar bisa dibuka kapan saja dari tombol ini."],
      ["#teamBtn", "Settings", "Pengaturan: folder data, scanner, salinan cadangan, dan kalibrasi skala."],
    ],
    tourNext: "Berikutnya →", tourEnd: "Selesai", tourSkip: "Lewati", tourHide: "Jangan tampilkan lagi", tourReplay: "Mulai tur aplikasi",
    gTitle: "Panduan pemakaian", gPrev: "← Sebelumnya", gNext: "Berikutnya →",
    guide: [
      ["Isi data pasien", "Isi usia, jenis kelamin, dan inisial operator. Tidak perlu nama, ID dibuat otomatis. Tekan <b>Mulai pindai</b>."],
      ["Letakkan kaki", "Pasien duduk. Letakkan kaki di tengah kaca, tumit beberapa cm dari tepi. Jari kaki rileks, jangan menekuk."],
      ["Tutup dengan kain hitam", "Tutupi kaki dan betis dengan kain hitam sampai seluruh kaca tertutup dan tidak ada cahaya masuk dari bawah. Jangan ada kain di bawah telapak."],
      ["Pindai", "Tekan <b>Mulai pindai</b>. Kaki tetap diam sekitar 20 detik sampai pemindaian selesai."],
      ["Tinjau", "Pastikan garis biru mengikuti bentuk kaki. Tekan <b>Ulangi</b> jika jari menekuk, kaki terpotong di tepi, atau ada peringatan. Jika sudah baik, tekan <b>Terima</b>."],
      ["Selesai", "Setelah semua kaki dipindai, sesi tersimpan otomatis dan disalin ke folder cadangan. Tekan <b>Sesi baru</b> untuk pasien berikutnya."],
    ],
  },
  en: {
    guideBtn: "? Guide", outline: "Outline", more: "More details", replay: "↻ Replay", close: "Close",
    prec: "Precision", basis: "Basis", refs: "References", scaleH: "How the scale works", noMeas: "No numbers available for this scan.",
    detNote: "Reference points can be adjusted in Measurement details after the session ends.",
    rulerCap: "Scan of the steel ruler under the white cloth (13 Nov 2026). Orange marks = every 11.84 pixels (1 mm), blue line = 10 mm = 118 pixels. The orange marks land right on the ruler's lines.",
    calSteps: ["Ruler on the acrylic", "Cover with the white cloth", "Scan, then count pixels"],
    scale: "The scanner turns the image into dots (pixels). We measured a steel ruler on the glass: 1 mm = 11.84 pixels lengthwise and 11.79 pixels across. The app counts the foot's pixels and converts them to mm. Checked 13 Nov 2026.",
    tape: `Average of 8 scans ${TAPE.mean} mm vs tape measure ${TAPE.tape} mm (difference ${(TAPE.tape - TAPE.mean).toFixed(1)} mm).`,
    i_fl: ["Foot length", "Distance from the back of the heel to the tip of the longest toe, measured along the foot axis (the line from the heel centre to the 2nd-toe tip)."],
    i_fw: ["Foot width", "The widest part of the sole, measured across the foot axis. Usually at the ball of the foot."],
    i_ai: ["Arch Index (AI)", "Compares the area of the middle of the sole (B) with the whole sole without toes (A + B + C): AI = B ÷ (A + B + C) [1]. A higher AI means more of the arch touches the glass.",
      "The app gives no flat / normal / high label. The limits in the literature were made for standing feet [1], while this measurement is seated."],
    i_area: ["Contact area", "Area of the whole sole touching the glass, toes included."],
    i_toeless: ["Area without toes", "Area of the sole from the heel to the base of the toes, toes excluded. This is what is split into thirds (A, B, C) for the Arch Index [1]."],
    i_A: ["Hindfoot (A)", "The back third of the sole without toes: the heel [1]."],
    i_B: ["Midfoot (B)", "The middle third of the sole without toes: the arch [1]."],
    i_C: ["Forefoot (C)", "The front third of the sole without toes: the ball of the foot [1]."],
    i_csi: ["Chippaux-Smirak (%)", "The narrowest width of the middle (arch) divided by the widest width of the front, times 100 [1]. Higher means a wider part of the arch touches the glass."],
    tour: [
      ["#steps", "Workflow", "Every patient goes through 4 steps: Start, Scan, Review, Done. The active step is white."],
      ["#schip", "Scanner status", "Green = the scanner is ready. Amber or red = something needs attention, tap to check again."],
      ["#main .card", "Patient details", "Enter age, sex and operator initials. No name needed. The ID is made automatically."],
      ["#guideBtn", "Guide", "The full steps with pictures can be opened any time from this button."],
      ["#teamBtn", "Settings", "Settings: data folder, scanner, backup copy and scale calibration."],
    ],
    tourNext: "Next →", tourEnd: "Done", tourSkip: "Skip", tourHide: "Don't show again", tourReplay: "Start the app tour",
    gTitle: "How to use", gPrev: "← Previous", gNext: "Next →",
    guide: [
      ["Enter patient details", "Enter age, sex and operator initials. No name needed, the ID is made automatically. Press <b>Start scanning</b>."],
      ["Place the foot", "The patient sits. Place the foot in the middle of the glass, heel a few cm from the edge. Toes relaxed, not curled."],
      ["Cover with the black cloth", "Cover the foot and lower leg with the black cloth until the whole glass is covered and no light gets in from below. No cloth under the sole."],
      ["Scan", "Press <b>Start scan</b>. The foot stays still for about 20 seconds until the scan ends."],
      ["Review", "Check that the blue line follows the foot. Press <b>Retake</b> if the toes are curled, the foot is cut off at the edge, or there is a warning. If it looks right, press <b>Accept</b>."],
      ["Done", "When every foot is scanned, the session is saved automatically and copied to the backup folder. Press <b>New session</b> for the next patient."],
    ],
  },
};
const x = (k) => X[L][k];
const num = (v, d) => (v == null || !isFinite(v) ? "–" : L === "id" ? Number(v).toFixed(d).replace(".", ",") : Number(v).toFixed(d));

// value, decimals, unit for each key from a Measurements object
const VAL = {
  fl: (m) => [m.fl_mm, 1, "mm"], fw: (m) => [m.fw_mm, 1, "mm"], ai: (m) => [m.ai, 3, ""], area: (m) => [m.area_cm2, 1, "cm²"],
  toeless: (m) => [m.toeless_area_cm2, 1, "cm²"], A: (m) => [m.hind_cm2, 1, "cm²"], B: (m) => [m.mid_cm2, 1, "cm²"], C: (m) => [m.fore_cm2, 1, "cm²"],
  csi: (m) => [m.csi_pct, 1, ""],
};
const precText = (k) => { const [p, u] = PREC[k]; return `± ${num(p, k === "ai" ? 3 : 1)}${u ? " " + u : ""}`; };
const ib = (k) => `<button class="ib" data-i="${k}" type="button" aria-label="info">i</button>`;

// ---------- the 3 main values + fold-out, used on the review screen ----------
function metricsBlock(m) {
  if (!m) return `<p class="muted" style="margin-top:18px">${x("noMeas")}</p>`;
  const big = (k) => { const [v, d, u] = VAL[k](m); return `<div class="met"><div class="ml">${esc(x("i_" + k)[0])}${ib(k)}</div><div class="mv">${num(v, d)}${u ? ` <small>${u}</small>` : ""}</div><div class="mp">${precText(k)}</div></div>`; };
  const row = (k) => { const [v, d, u] = VAL[k](m); return `<div class="kv"><span>${esc(x("i_" + k)[0])}${ib(k)}</span><b>${num(v, d)}${u ? " " + u : ""} <em class="mp2">${precText(k)}</em></b></div>`; };
  return `<div class="mets">${big("fl")}${big("fw")}${big("ai")}</div>
    <details class="more"><summary>${x("more")}</summary>${["area", "toeless", "A", "B", "C", "csi"].map(row).join("")}
      <p class="muted" style="margin:10px 0 0">${x("detNote")}</p></details>`;
}
// wire the ⓘ buttons inside `root`; ctx = { img, w, h, down, m } for the animations
function wireInfo(root, ctx) {
  root.querySelectorAll(".ib").forEach((b) => (b.onclick = (e) => { e.preventDefault(); e.stopPropagation(); openInfo(b.dataset.i, ctx); }));
}

// ---------- modal ----------
function modal(html, cls = "") {
  closeModal();
  const d = document.createElement("div");
  d.className = "modal"; d.id = "modal";
  d.innerHTML = `<div class="mbox ${cls}" role="dialog">${html}</div>`;
  d.onclick = (e) => { if (e.target === d) closeModal(); };
  document.body.append(d);
  d.querySelectorAll("[data-close]").forEach((b) => (b.onclick = closeModal));
  return d;
}
function closeModal() { $("#modal")?.remove(); }
document.addEventListener("keydown", (e) => { if (e.key === "Escape") { closeModal(); endTour(false); } });

function openInfo(k, ctx) {
  const [title, body, extra] = x("i_" + k);
  const m = ctx?.m;
  // each pop-up numbers its own references from [1]: CSI cites Papuga & Burke, the rest Cavanagh & Rodgers
  const cited = /\[1\]/.test(body + (extra || ""));
  const refs = !cited ? [] : k === "csi" ? [REFS.pap] : [REFS.cav];
  const anim = m && ["fl", "fw", "ai", "toeless", "A", "B", "C"].includes(k);
  const val = m ? (() => { const [v, d, u] = VAL[k](m); return `<div class="mv" style="margin:2px 0 12px">${num(v, d)}${u ? ` <small>${u}</small>` : ""}</div>`; })() : "";
  const scaleTxt = ["fl", "fw", "area"].includes(k) ? `<h4>${x("scaleH")}</h4><p>${x("scale")}</p>${rulerFig()}` : "";
  const d = modal(`<div class="mhead"><h3>${esc(title)}</h3><button class="btn ghost mx" data-close>✕</button></div>
    <div class="mcols">${anim ? `<div class="viz"><div id="vizbox"></div><button class="btn ghost" id="replay" style="padding:8px 14px;font-size:14px;margin-top:8px">${x("replay")}</button></div>` : ""}
    <div class="mtext">${val}<p>${body}</p>${extra ? `<p>${extra}</p>` : ""}
      ${PREC[k] ? `<h4>${x("prec")}</h4><p><b>${precText(k)}</b> · ${x("basis")}: ${PREC_SRC[L]}.${k === "fl" ? " " + x("tape") : ""}</p>` : ""}
      ${scaleTxt}
      ${refs.length ? `<h4>${x("refs")}</h4><ol class="refs">${refs.map((r) => `<li>${r}</li>`).join("")}</ol>` : ""}</div></div>`, "wide");
  if (anim) {
    const play = () => { $("#vizbox").innerHTML = vizSvg(k, ctx); };
    play(); $("#replay").onclick = play;
  }
  italicize(d);
}

// ---------- animations drawn on the patient's own scan ----------
function vizSvg(k, ctx) {
  const { m, w, h, down, img } = ctx;
  const P = (q) => [q[0] / down, q[1] / down];
  const H = P(m.heel_centre), J = P(m.toe_tip), E = P(m.heel_end);
  const l = Math.hypot(J[0] - H[0], J[1] - H[1]) || 1, a = [(J[0] - H[0]) / l, (J[1] - H[1]) / l], b = [-a[1], a[0]];
  const ppm = 1 / (m.scale.y_mm_per_px * down);
  const FL = m.fl_mm * ppm, Lt = m.toeless_frac * m.fl_mm * ppm, half = (m.fw_mm / 2) * 1.2 * ppm;
  const at = (p, s, t = 0) => [p[0] + a[0] * s + b[0] * t, p[1] + a[1] * s + b[1] * t];
  const T = at(E, FL);
  // crop around the foot
  const pts = [E, T, at(E, 0, half), at(E, 0, -half), at(T, 0, half), at(T, 0, -half)];
  const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]), pad = 30;
  const x0 = Math.max(0, Math.min(...xs) - pad), y0 = Math.max(0, Math.min(...ys) - pad);
  const x1 = Math.min(w, Math.max(...xs) + pad), y1 = Math.min(h, Math.max(...ys) + pad);
  const vb = `${x0} ${y0} ${x1 - x0} ${y1 - y0}`, sw = (x1 - x0) / 120; // stroke width relative to crop
  const line = (p, q, cls, extra = "") => `<line class="${cls}" x1="${p[0]}" y1="${p[1]}" x2="${q[0]}" y2="${q[1]}" ${extra}/>`;
  const len = (p, q) => Math.hypot(q[0] - p[0], q[1] - p[1]);
  const label = (p, txt, cls) => `<text class="${cls}" x="${p[0]}" y="${p[1]}" font-size="${sw * 7}" text-anchor="middle" dominant-baseline="middle">${txt}</text>`;
  let g = "";
  const arrowDefs = `<defs><marker id="ah" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="4" markerHeight="4" orient="auto-start-reverse"><path d="M0,0L10,5L0,10z" fill="#ffd166"/></marker></defs>`;
  if (k === "fl" || k === "fw") {
    const [p, q] = k === "fl" ? [E, T] : [P(m.fw_line[0]), P(m.fw_line[1])];
    const L0 = len(p, q), mid = [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2], off = k === "fl" ? at(mid, 0, -half * 0.9) : at(mid, -sw * 14);
    g += line(p, q, "draw", `stroke="#ffd166" stroke-width="${sw * 1.6}" marker-start="url(#ah)" marker-end="url(#ah)" style="stroke-dasharray:${L0};stroke-dashoffset:${L0}"`);
    g += label(off, `${num(k === "fl" ? m.fl_mm : m.fw_mm, 1)} mm`, "lab s3");
  } else {
    const hl = { ai: "B", toeless: "all", A: "A", B: "B", C: "C" }[k];
    // 1 heel centre + toe tip, 2 the axis, 3 the cuts, 4 the thirds, 5 the formula
    g += `<circle class="pop s1" cx="${H[0]}" cy="${H[1]}" r="${sw * 2.4}" fill="#fff" stroke="#1f5fbf" stroke-width="${sw}"/><circle class="pop s1" cx="${J[0]}" cy="${J[1]}" r="${sw * 2.4}" fill="#fff" stroke="#1f5fbf" stroke-width="${sw}"/>`;
    const ax = len(E, J);
    g += line(E, J, "draw s2", `stroke="#fff" stroke-width="${sw * 0.9}" stroke-dasharray="${ax}" style="stroke-dashoffset:${ax}"`);
    const names = ["A", "B", "C"], fills = { A: "#5aa0ff", B: "#ffd166", C: "#3ec58d" };
    for (let i = 0; i < 3; i++) {
      const s0 = (Lt * i) / 3, s1 = (Lt * (i + 1)) / 3, n = names[i], on = hl === "all" || hl === n;
      const poly = [at(E, s0, -half), at(E, s1, -half), at(E, s1, half), at(E, s0, half)].map((p) => p.join(",")).join(" ");
      g += `<polygon class="fade s4" points="${poly}" fill="${fills[n]}" style="--o:${on ? 0.5 : 0.14}"/>`;
      g += label(at(E, (s0 + s1) / 2, half * 1.18), n, "lab s4");
    }
    for (let i = 0; i <= 3; i++) g += line(at(E, (Lt * i) / 3, -half), at(E, (Lt * i) / 3, half), "fade s3", `stroke="${i == 3 ? "#ffd166" : "#fff"}" stroke-width="${sw * (i == 3 ? 1 : 0.7)}" stroke-dasharray="${sw * 3} ${sw * 2}" style="--o:.9"`);
  }
  const formula = k === "ai" ? `<div class="formula s5">AI = B ÷ (A + B + C) = ${num(m.mid_cm2, 1)} ÷ ${num(m.toeless_area_cm2, 1)} = <b>${num(m.ai, 3)}</b></div>` : "";
  return `<svg viewBox="${vb}" class="vizsvg">${arrowDefs}<image href="${img}" width="${w}" height="${h}"/>${g}</svg>${formula}`;
}

// ---------- calibration: real ruler crop + procedure animation ----------
const rulerFig = () => `<figure class="rfig"><img src="assets/ruler_ticks.jpg" alt=""><figcaption>${x("rulerCap")}</figcaption></figure>`;
function calibAnim() {
  const S = x("calSteps");
  const glass = (dx) => `<rect x="${dx + 10}" y="10" width="170" height="120" rx="8" fill="#10151f"/>`;
  const ruler = (dx, cls) => `<g class="${cls}"><rect x="${dx + 30}" y="56" width="130" height="20" rx="2" fill="#c9ced6"/>${Array.from({ length: 27 }, (_, i) => `<line x1="${dx + 34 + i * 4.7}" y1="56" x2="${dx + 34 + i * 4.7}" y2="${i % 5 ? 62 : 67}" stroke="#333" stroke-width="1"/>`).join("")}</g>`;
  return `<svg viewBox="0 0 590 168" class="calanim" role="img">
    <g>${glass(0)}${ruler(0, "ca-slide")}<text x="95" y="156" class="ca-t ca-t1">1 · ${S[0]}</text></g>
    <g>${glass(200)}${ruler(200, "")}<path class="ca-cloth" d="M205 18 Q240 8 285 20 T380 16 L384 132 Q330 122 290 134 T205 128 Z" fill="#fff"/><path class="ca-cloth" d="M230 30 Q260 70 250 120 M320 26 Q335 80 350 124" stroke="#d8dde6" stroke-width="3" fill="none"/><text x="295" y="156" class="ca-t ca-t2">2 · ${S[1]}</text></g>
    <g><rect x="410" y="10" width="170" height="120" rx="8" fill="#f4f6f9" stroke="#d8e0ec"/><rect x="430" y="56" width="130" height="20" rx="2" fill="#4a505a"/>${Array.from({ length: 27 }, (_, i) => `<line x1="${434 + i * 4.7}" y1="56" x2="${434 + i * 4.7}" y2="${i % 5 ? 62 : 67}" stroke="#d0d4da" stroke-width="1"/>`).join("")}
      <rect class="ca-bar" x="412" y="12" width="166" height="4" fill="#5aa0ff"/>
      <g class="ca-zoom"><circle cx="540" cy="34" r="22" fill="#fff" stroke="#1f5fbf" stroke-width="3"/>${[0, 1, 2, 3].map((i) => `<line x1="${525 + i * 10}" y1="25" x2="${525 + i * 10}" y2="38" stroke="#333" stroke-width="2.5"/>`).join("")}<line x1="525" y1="45" x2="535" y2="45" stroke="#ff8c00" stroke-width="3"/>
        <rect x="438" y="96" width="114" height="22" rx="11" fill="#1f5fbf"/><text x="495" y="111" text-anchor="middle" font-size="11" font-weight="700" fill="#fff">1 mm = ${num(11.84, 2)} px</text></g>
      <text x="495" y="156" class="ca-t ca-t3">3 · ${S[2]}</text></g></svg>`;
}

// ---------- guide (Panduan) ----------
const GUIDE_PIC = [
  // 1 form
  `<rect x="40" y="30" width="130" height="150" rx="10" fill="#fff" stroke="#1f5fbf" stroke-width="3"/>${[60, 90, 120].map((y) => `<rect x="58" y="${y}" width="94" height="16" rx="5" fill="#e8f0fb"/>`).join("")}<rect x="96" y="150" width="56" height="18" rx="6" fill="#1f5fbf"/>`,
  // 2 foot on glass
  `<rect x="30" y="20" width="150" height="170" rx="8" fill="#10151f"/><g transform="translate(24,-4) scale(.7)">${footShape("#d7a98a")}</g><line x1="30" y1="178" x2="180" y2="178" stroke="#ffd166" stroke-width="3" stroke-dasharray="6 5"/><text x="105" y="205" font-size="13" text-anchor="middle" fill="#6b7a90">↕ ± 3 cm</text>`,
  // 3 black cloth
  `<rect x="30" y="20" width="150" height="170" rx="8" fill="#10151f"/><g transform="translate(24,-4) scale(.7)">${footShape("#d7a98a")}</g><path d="M14 120 Q105 70 196 120 L196 205 L14 205 Z" fill="#222" opacity=".92"/><path d="M14 120 Q105 70 196 120" stroke="#555" stroke-width="3" fill="none"/>`,
  // 4 scan: bar
  `<rect x="30" y="20" width="150" height="170" rx="8" fill="#10151f"/><g transform="translate(24,-4) scale(.7)">${footShape("#d7a98a")}</g><rect class="scanbar" x="30" y="20" width="150" height="6" fill="#5aa0ff" opacity=".9"/><text x="105" y="210" font-size="14" text-anchor="middle" fill="#6b7a90">± 20 s</text>`,
  // 5 review: outline + buttons
  `<rect x="20" y="20" width="110" height="150" rx="8" fill="#10151f"/><g transform="translate(6,2) scale(.6)">${footShape("#d7a98a", "#1f5fbf")}</g><rect x="142" y="60" width="56" height="26" rx="8" fill="#fff" stroke="#c93b3b" stroke-width="2"/><text x="170" y="78" font-size="12" text-anchor="middle" fill="#c93b3b">↺</text><rect x="142" y="100" width="56" height="26" rx="8" fill="#1f9d6b"/><text x="170" y="118" font-size="13" text-anchor="middle" fill="#fff">✓</text>`,
  // 6 done
  `<circle cx="105" cy="100" r="62" fill="#e1f5ec"/><path d="M75 102 L98 125 L140 78" stroke="#1f9d6b" stroke-width="12" fill="none" stroke-linecap="round" stroke-linejoin="round"/>`,
];
function footShape(fill, stroke) {
  const toes = [[114, 62, 13, 16], [135, 70, 8, 11], [148, 82, 7, 10], [156, 97, 6, 9], [160, 112, 5, 8]];
  const s = stroke ? `stroke="${stroke}" stroke-width="3"` : "";
  return `<path d="${OUT}" fill="${fill}" ${s}/>${toes.map(([cx, cy, rx, ry]) => `<ellipse cx="${cx}" cy="${cy}" rx="${rx}" ry="${ry}" fill="${fill}" ${s}/>`).join("")}`;
}
let gi = 0;
function openGuide(start = 0) {
  gi = start;
  const G = x("guide");
  const paint = () => {
    const [h, p] = G[gi];
    modal(`<div class="mhead"><h3>${x("gTitle")}</h3><button class="btn ghost mx" data-close>✕</button></div>
      <div class="gstep"><svg viewBox="0 0 210 220" class="gpic">${GUIDE_PIC[gi]}</svg>
        <div><div class="gnum">${gi + 1} / ${G.length}</div><h4 style="font-size:24px;margin:4px 0 10px">${h}</h4><p style="font-size:18px;line-height:1.5">${p}</p></div></div>
      <div class="gdots">${G.map((_, i) => `<span class="${i === gi ? "on" : ""}" data-g="${i}"></span>`).join("")}</div>
      <div class="row" style="margin-top:16px"><button class="btn ghost" id="gprev" style="padding:12px 20px;font-size:16px" ${gi === 0 ? "disabled" : ""}>${x("gPrev")}</button>
        <button class="btn ghost" id="gtour" style="padding:12px 20px;font-size:16px">${x("tourReplay")}</button>
        <button class="btn right" id="gnext" style="padding:12px 24px;font-size:16px">${gi === G.length - 1 ? x("close") : x("gNext")}</button></div>`, "wide");
    $("#gprev").onclick = () => { gi--; paint(); };
    $("#gnext").onclick = () => { if (gi === G.length - 1) closeModal(); else { gi++; paint(); } };
    $("#gtour").onclick = () => { closeModal(); startTour(true); };
    document.querySelectorAll("[data-g]").forEach((d) => (d.onclick = () => { gi = Number(d.dataset.g); paint(); }));
    italicize($("#modal"));
  };
  paint();
}

// ---------- first-run tour (once per laptop until "don't show again" is ticked) ----------
let tour = null;
function startTour(force) {
  if (!force && localStorage.getItem("tourDone") === "1") return;
  if (st.screen !== "start") { st.screen = "start"; render(); }
  tour = { i: 0 };
  paintTour();
}
function endTour(save) {
  if (!tour) return;
  if (save) localStorage.setItem("tourDone", "1");
  tour = null; $("#tour")?.remove();
}
function paintTour() {
  $("#tour")?.remove();
  if (!tour) return;
  const S = x("tour"), [sel, h, p] = S[tour.i], el = document.querySelector(sel);
  if (!el) { tour.i < S.length - 1 ? (tour.i++, paintTour()) : endTour(false); return; }
  const r = el.getBoundingClientRect(), last = tour.i === S.length - 1;
  const d = document.createElement("div"); d.id = "tour";
  const below = r.bottom + 230 < innerHeight;
  const left = Math.min(Math.max(12, r.left), innerWidth - 392);
  d.innerHTML = `<div class="spot" style="left:${r.left - 8}px;top:${r.top - 8}px;width:${r.width + 16}px;height:${r.height + 16}px"></div>
    <div class="tip" style="left:${left}px;${below ? `top:${r.bottom + 16}px` : `top:${Math.max(12, r.top - 216)}px`}">
      <div class="gnum">${tour.i + 1} / ${S.length}</div><h4>${h}</h4><p>${p}</p>
      <label class="hide"><input type="checkbox" id="thide" ${tour.hide ? "checked" : ""}> ${x("tourHide")}</label>
      <div class="row" style="margin-top:12px"><button class="btn ghost" id="tskip" style="padding:9px 16px;font-size:15px">${x("tourSkip")}</button>
      <button class="btn right" id="tnext" style="padding:9px 18px;font-size:15px">${last ? x("tourEnd") : x("tourNext")}</button></div></div>`;
  document.body.append(d);
  $("#thide").onchange = (e) => (tour.hide = e.target.checked);
  $("#tskip").onclick = () => endTour(tour.hide);
  $("#tnext").onclick = () => { if (last) endTour(tour.hide); else { tour.i++; paintTour(); } };
  italicize(d);
}
addEventListener("resize", () => tour && paintTour());

// ---------- logo + opening animation ----------
const LOGO_TOES = `<ellipse cx="31" cy="39" rx="9.5" ry="12" transform="rotate(-8 31 39)"/><circle cx="48" cy="29" r="6.2"/><circle cx="61" cy="31" r="5.6"/><circle cx="72" cy="38" r="5"/><circle cx="80" cy="48" r="4.4"/>`;
const LOGO_SOLE = `<path d="M52 192 C36 192 30 178 31 160 C32 142 41 132 40 117 C39 104 22 92 23 75 C24 58 44 52 60 54 C78 56 84 74 80 94 C76 112 69 124 70 146 C71 168 72 192 52 192 Z"/>`;
const LOGO = (cls = "") => `<svg viewBox="14 18 76 178" class="${cls}" aria-hidden="true"><g fill="currentColor">${LOGO_TOES}${LOGO_SOLE}</g></svg>`;
// opening logo: a scan bar sweeps down and reveals the footprint as scan lines, which then fill in
function splashLogo() {
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) return LOGO("sp-logo");
  const shapes = LOGO_TOES + LOGO_SOLE;
  return `<svg viewBox="14 18 76 178" class="sp-logo" aria-hidden="true"><defs>
    <clipPath id="spclip"><rect x="0" y="16" width="110" height="0"><animate attributeName="height" from="0" to="182" begin="0.15s" dur="1.1s" fill="freeze"/></rect></clipPath>
    <pattern id="spscan" width="4" height="3.2" patternUnits="userSpaceOnUse"><rect width="4" height="1.7" fill="#fff"/></pattern></defs>
    <g clip-path="url(#spclip)"><g fill="url(#spscan)">${shapes}</g>
      <g fill="currentColor" opacity="0">${shapes}<animate attributeName="opacity" from="0" to="1" begin="1.25s" dur="0.45s" fill="freeze"/></g></g>
    <rect x="6" y="15" width="92" height="2.6" rx="1.3" fill="#a9dcff" opacity="0">
      <animate attributeName="y" from="15" to="196" begin="0.15s" dur="1.1s" fill="freeze"/>
      <animate attributeName="opacity" values="0;1;1;0" keyTimes="0;0.06;0.9;1" begin="0.15s" dur="1.15s" fill="freeze"/></rect></svg>`;
}
function splash() {
  const d = document.createElement("div");
  d.id = "splash";
  d.innerHTML = `<div class="sp-in">${splashLogo()}<div class="sp-name">Smart<span>Foot</span> Panorama</div></div>`;
  document.body.append(d);
  const done = () => { if (!d.isConnected) return; d.classList.add("out"); setTimeout(() => { d.remove(); if (st.screen !== "setup") startTour(false); }, 380); };
  d.onclick = done;
  setTimeout(done, matchMedia("(prefers-reduced-motion: reduce)").matches ? 600 : 2300);
}

// top bar: logo next to the brand, guide button
$("aside .brand").insertAdjacentHTML("afterbegin", LOGO("brand-logo"));
$("aside .row").insertAdjacentHTML("afterbegin", `<button class="lang" id="guideBtn" type="button"></button>`);
$("#guideBtn").onclick = () => openGuide(0);
const paintGuideBtn = () => { $("#guideBtn").textContent = x("guideBtn"); };
paintGuideBtn();
$("#lang").addEventListener("click", paintGuideBtn);
splash();

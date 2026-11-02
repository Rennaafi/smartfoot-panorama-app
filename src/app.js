// SmartFoot front end. All hardware, file and image work happens in Rust; this file only draws
// screens and calls the commands in src-tauri/src/lib.rs.
const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args);

const T = {
  id: {
    steps: ["Mulai", "Pindai", "Tinjau", "Selesai"], team: "⚙ Tim",
    start: "Sesi baru", startSub: "Isi data singkat pasien. Tidak perlu nama.",
    pid: "ID pasien", age: "Usia", sex: "Jenis kelamin", f: "Perempuan", m: "Laki-laki",
    op: "Inisial operator", opHint: "wajib diisi", note: "Catatan (opsional)", go: "Mulai pindai", starting: "Memeriksa pemindai…",
    scan: "Pindai kaki", scanSub: "Letakkan kaki di atas kaca, tumit pada penahan. Diam sampai pemindaian selesai.",
    left: "Kaki kiri", right: "Kaki kanan", putaran: "Putaran", doScan: "Mulai pindai", scanning: "Memindai… jangan gerakkan kaki", ready: "Siap",
    review: "Tinjau hasil", reviewSub: "Garis biru = tepi kaki yang terdeteksi. Pastikan sesuai bentuk kaki.",
    chkFound: "Kaki terdeteksi", chkInside: "Kaki penuh di dalam area",
    wNoFoot: "Kaki tidak terdeteksi. Pastikan kaki menempel di kaca, lalu ulangi.",
    wEdge: "Kaki menyentuh tepi gambar. Geser kaki menjauhi tepi, lalu ulangi.",
    accept: "Terima", acceptAnyway: "Tetap terima", retake: "Ulangi", saving: "Menyimpan…",
    done: "Sesi selesai", doneSub: "Semua pemindaian tersimpan.", saved: "Tersimpan di", scans: "pemindaian", newS: "Sesi baru",
    set: "Pengaturan tim", setSub: "Hanya untuk tim peneliti. Perawat tidak perlu membuka halaman ini.",
    teamOnly: "Area tim · tidak ditampilkan di alur perawat",
    rnd: "Jumlah putaran bawaan", r1: "1 putaran", r3: "3 putaran", rndNote: "Tercatat di setiap file sesi.",
    dataf: "Folder data", save: "Simpan", saved2: "Tersimpan ✓",
    scanner: "Pemindai", auto: "Otomatis (LiDE 300)", refresh: "Segarkan daftar",
    cal: "Kalibrasi skala", calNote: "Alat kalibrasi belum dibuat. Sementara semua sesi dicatat tanpa kalibrasi (anggap 300 dpi nominal).",
    back: "← Kembali", cancel: "Batalkan sesi", cancelSure: "Yakin? Tekan lagi untuk batal",
    e_NOT_FOUND: "Pemindai tidak ditemukan. Periksa kabel USB dan pastikan pemindai menyala.",
    e_LOCKED: "Gambar kosong: kunci transportasi pemindai masih tertutup. Buka kunci (LOCK) lalu ulangi.",
    e_BUSY: "Pemindai sedang sibuk. Tunggu beberapa detik lalu coba lagi.",
    e_SCAN_FAILED: "Pemindaian gagal", e_ID_USED: "ID ini sudah dipakai. Gunakan ID lain.", e_EMPTY_ID: "ID pasien kosong.", e_OP: "Isi inisial operator.",
    retry: "Coba lagi",
  },
  en: {
    steps: ["Start", "Scan", "Review", "Done"], team: "⚙ Team",
    start: "New session", startSub: "Enter brief patient details. No name needed.",
    pid: "Patient ID", age: "Age", sex: "Sex", f: "Female", m: "Male",
    op: "Operator initials", opHint: "required", note: "Note (optional)", go: "Start scanning", starting: "Checking the scanner…",
    scan: "Scan foot", scanSub: "Place the foot on the glass, heel against the stop. Keep still until the scan ends.",
    left: "Left foot", right: "Right foot", putaran: "Round", doScan: "Start scan", scanning: "Scanning… keep the foot still", ready: "Ready",
    review: "Review result", reviewSub: "Blue line = detected foot edge. Make sure it follows the foot.",
    chkFound: "Foot detected", chkInside: "Whole foot inside the area",
    wNoFoot: "No foot detected. Make sure the foot rests on the glass, then retake.",
    wEdge: "The foot touches the image edge. Move it away from the edge, then retake.",
    accept: "Accept", acceptAnyway: "Accept anyway", retake: "Retake", saving: "Saving…",
    done: "Session complete", doneSub: "All scans are saved.", saved: "Saved to", scans: "scans", newS: "New session",
    set: "Team settings", setSub: "For the research team only. Nurses never need this page.",
    teamOnly: "Team area · not shown in the nurse flow",
    rnd: "Default number of rounds", r1: "1 round", r3: "3 rounds", rndNote: "Recorded in every session file.",
    dataf: "Data folder", save: "Save", saved2: "Saved ✓",
    scanner: "Scanner", auto: "Automatic (LiDE 300)", refresh: "Refresh list",
    cal: "Scale calibration", calNote: "The calibration tool is not built yet. Until then every session is recorded without calibration (nominal 300 dpi assumed).",
    back: "← Back", cancel: "Cancel session", cancelSure: "Sure? Press again to cancel",
    e_NOT_FOUND: "Scanner not found. Check the USB cable and that the scanner is switched on.",
    e_LOCKED: "Blank image: the scanner's transport lock is still closed. Open the LOCK switch and retake.",
    e_BUSY: "The scanner is busy. Wait a few seconds and try again.",
    e_SCAN_FAILED: "Scan failed", e_ID_USED: "This ID is already used. Pick another ID.", e_EMPTY_ID: "The patient ID is empty.", e_OP: "Enter the operator initials.",
    retry: "Try again",
  },
};

let L = localStorage.getItem("lang") || "id";
const t = (k) => T[L][k];
const $ = (s) => document.querySelector(s);
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));

const st = { screen: "start", prev: "start", form: { pid: "", op: "", age: "", sex: "f", note: "" }, sess: null, idx: 0, scan: null, error: "", busy: false, settings: null, scanners: [] };
const STEP = { start: 0, scan: 1, review: 2, done: 3 };

function errText(raw) {
  const m = String(raw).match(/^([A-Z_]+):\s*(.*)$/s);
  if (!m) return esc(raw);
  const known = T[L]["e_" + m[1]];
  return esc(known ? (m[1] === "SCAN_FAILED" ? `${known}: ${m[2]}` : known) : raw);
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

async function render() {
  steps();
  const m = $("#main");
  $("#teamBtn").textContent = t("team");
  const s = st.screen;

  if (s === "start") {
    if (!st.form.pid) { try { st.form.pid = await invoke("new_session_id"); } catch (e) { st.error = e; } }
    const f = st.form;
    m.innerHTML = `<h1>${t("start")}</h1><p class="sub">${t("startSub")}</p>${errBox()}<div class="card"><div class="grid2">
      <div class="field"><label>${t("pid")}</label><input id="f_pid" value="${esc(f.pid)}"></div>
      <div class="field"><label>${t("op")} <span class="muted">(${t("opHint")})</span></label><input id="f_op" value="${esc(f.op)}" placeholder="RF" maxlength="4"></div>
      <div class="field"><label>${t("age")}</label><input id="f_age" type="number" min="0" max="120" value="${esc(f.age)}" placeholder="45"></div>
      <div class="field"><label>${t("sex")}</label><div class="seg"><button class="${f.sex == "f" ? "on" : ""}" data-sex="f">${t("f")}</button><button class="${f.sex == "m" ? "on" : ""}" data-sex="m">${t("m")}</button></div></div>
      <div class="field" style="grid-column:1/-1"><label>${t("note")}</label><input id="f_note" value="${esc(f.note)}"></div></div>
      <div class="row"><button class="btn right" id="go" ${st.busy ? "disabled" : ""}>${st.busy ? t("starting") : t("go") + " →"}</button></div></div>`;
    const keep = () => { f.pid = $("#f_pid").value; f.op = $("#f_op").value; f.age = $("#f_age").value; f.note = $("#f_note").value; };
    m.querySelectorAll("[data-sex]").forEach((b) => (b.onclick = () => { keep(); f.sex = b.dataset.sex; render(); }));
    $("#go").onclick = async () => {
      keep(); st.error = "";
      if (!f.op.trim()) { m.querySelector("h1").insertAdjacentHTML("afterend", `<div class="err">${t("e_OP")}</div>`); return; }
      st.busy = true; render();
      try {
        const info = await invoke("start_session", { pid: f.pid, operator: f.op, age: f.age === "" ? null : Number(f.age), sex: f.sex, note: f.note });
        st.sess = { ...info, seq: seqOf(info.rounds) };
        st.idx = 0; st.scan = null; st.screen = "scan";
      } catch (e) { st.error = e; }
      st.busy = false; render();
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
        st.scan = await invoke("scan_foot", { pid: st.sess.pid, side, round: r });
        clearInterval(iv); st.screen = "review";
      } catch (e) { clearInterval(iv); st.error = e; }
      render();
    };
  }

  if (s === "review") {
    const [side, r] = st.sess.seq[st.idx], sc = st.scan, w = sc.warnings;
    const noFoot = w.includes("NO_FOOT"), edge = w.includes("EDGE"), ok = !w.length;
    const warnText = [noFoot ? t("wNoFoot") : "", edge ? t("wEdge") : ""].filter(Boolean).join(" ");
    m.innerHTML = `<h1>${t("review")} · ${t(side)} ${r}/${st.sess.rounds}</h1><p class="sub">${t("reviewSub")}</p>${errBox()}<div class="grid2"><div class="card"><div class="scanwin" style="background:#10151f"><img src="${sc.preview}" alt=""></div></div>
      <div class="card"><ul class="checks"><li><span class="dot ${noFoot ? "bad" : ""}"></span>${t("chkFound")}</li><li><span class="dot ${edge || noFoot ? "bad" : ""}"></span>${t("chkInside")}</li></ul>
      ${ok ? "" : `<div class="warn">${warnText}</div>`}
      <div class="row" style="margin-top:24px"><button class="btn ${ok ? "bad" : ""}" id="retake">↺ ${t("retake")}</button>
      <button class="btn ${ok ? "ok" : "ghost"} right" id="accept">${ok ? "✓ " + t("accept") : t("acceptAnyway")}</button></div>${cancelBtn()}</div></div>`;
    wireCancel();
    $("#retake").onclick = async () => {
      try { await invoke("retake_scan", { pid: st.sess.pid, side, round: r }); st.scan = null; st.screen = "scan"; st.error = ""; } catch (e) { st.error = e; }
      render();
    };
    $("#accept").onclick = async () => {
      $("#accept").disabled = true; $("#accept").textContent = t("saving");
      try {
        await invoke("accept_scan", { pid: st.sess.pid, side, round: r, seconds: sc.seconds, widthPx: sc.width_px, heightPx: sc.height_px, warnings: w });
        if (st.idx < st.sess.seq.length - 1) { st.idx++; st.screen = "scan"; }
        else { await invoke("finish_session", { pid: st.sess.pid }); st.screen = "done"; }
        st.scan = null; st.error = "";
      } catch (e) { st.error = e; }
      render();
    };
  }

  if (s === "done") {
    m.innerHTML = `<h1>${t("done")}</h1><p class="sub">${t("doneSub")}</p><div class="card"><div class="row" style="gap:30px"><div class="big" style="color:var(--ok)">✓</div><div><div style="font-size:22px;font-weight:700">${esc(st.sess.pid)} · ${st.sess.seq.length} ${t("scans")}</div><div class="sub" style="margin:4px 0 0">${t("saved")} ${esc(st.sess.dir)}</div></div></div>
      <div class="row" style="margin-top:26px"><button class="btn right" id="new">${t("newS")}</button></div></div>`;
    $("#new").onclick = () => { st.form = { pid: "", op: st.form.op, age: "", sex: "f", note: "" }; st.sess = null; st.screen = "start"; render(); };
  }

  if (s === "settings") {
    const c = st.settings;
    m.innerHTML = `<div class="teambar">🔒 ${t("teamOnly")}</div><h1>${t("set")}</h1><p class="sub">${t("setSub")}</p>${errBox()}<div class="grid2">
      <div class="card"><label>${t("rnd")}</label><div class="seg"><button class="${c.rounds == 1 ? "on" : ""}" data-r="1">${t("r1")}</button><button class="${c.rounds == 3 ? "on" : ""}" data-r="3">${t("r3")}</button></div><p class="muted" style="margin:8px 0 22px">${t("rndNote")}</p>
        <label>${t("dataf")}</label><div class="row"><input id="s_dir" value="${esc(c.data_dir)}" style="flex:1"><button class="btn ghost" id="s_save" style="padding:12px 18px;font-size:16px">${t("save")}</button></div>
        <label style="margin-top:22px">${t("scanner")}</label><div class="row"><select id="s_dev" style="flex:1"><option value="">${t("auto")}</option>${st.scanners.map((d) => `<option ${c.scanner == d ? "selected" : ""}>${esc(d)}</option>`).join("")}</select><button class="btn ghost" id="s_ref" style="padding:12px 18px;font-size:16px">${t("refresh")}</button></div></div>
      <div class="card"><h2 style="font-size:20px;margin-bottom:6px">${t("cal")}</h2><p class="muted">${t("calNote")}</p></div></div>
      <div class="row" style="margin-top:22px"><button class="btn ghost" id="back">${t("back")}</button></div>`;
    const persist = async (patch) => { try { st.settings = await invoke("set_settings", { new: { ...st.settings, ...patch } }); st.error = ""; } catch (e) { st.error = e; } render(); };
    m.querySelectorAll("[data-r]").forEach((b) => (b.onclick = () => persist({ rounds: Number(b.dataset.r) })));
    $("#s_save").onclick = () => persist({ data_dir: $("#s_dir").value.trim() });
    $("#s_dev").onchange = (e) => persist({ scanner: e.target.value || null });
    $("#s_ref").onclick = async () => { try { st.scanners = await invoke("list_scanners"); } catch (e) { st.error = e; } render(); };
    $("#back").onclick = () => { st.screen = st.prev; st.error = ""; render(); };
  }
}

$("#lang").onclick = () => { L = L == "id" ? "en" : "id"; localStorage.setItem("lang", L); render(); };
$("aside .row").insertAdjacentHTML("afterbegin", `<button class="lang" id="teamBtn"></button>`);
$("#teamBtn").onclick = async () => {
  if (st.screen === "settings") { st.screen = st.prev; }
  else {
    if (st.busy) return;
    st.prev = st.screen; st.screen = "settings"; st.error = "";
    try { st.settings = await invoke("get_settings"); st.scanners = await invoke("list_scanners").catch(() => []); } catch (e) { st.error = e; }
  }
  render();
};
render();

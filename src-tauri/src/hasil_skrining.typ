// The patient's result sheet ("Hasil skrining kaki"), two A4 pages, filled from data.json by pdf.rs.
// Design = version D agreed on 26 Nov 2026 (mockups/v031_style/varD.typ). Text is Bahasa Indonesia.
#let d = json("data.json")
#let c = d.content
#let blue = rgb("#1f5fbf")
#let navy = rgb("#0f2a5c")
#let ink = rgb("#14213d")
#let mute = rgb("#4f5d73")
#let accent = rgb("#c2561a")
#let red = rgb("#d23c3c")
#let hair = 0.6pt + rgb("#e3e9f2")
#let edge = 0.6pt + rgb("#dbe4f0")
#let risk = (rgb("#2e9e5b"), rgb("#7cb342"), rgb("#e8a317"), rgb("#e46f22"), rgb("#d23c3c"))
#let riskdark = (rgb("#1d6e3e"), rgb("#4f7d1f"), rgb("#8a5c00"), rgb("#9c4310"), rgb("#9e2222"))
#let script-font = "Segoe Script"
#let peny = if d.penyelenggara != "" { d.penyelenggara } else { "Tim SmartFoot" }

#let foot-svg(col) = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='14 18 76 178'><g fill='" + col + "'><ellipse cx='31' cy='39' rx='9.5' ry='12' transform='rotate(-8 31 39)'/><circle cx='48' cy='29' r='6.2'/><circle cx='61' cy='31' r='5.6'/><circle cx='72' cy='38' r='5'/><circle cx='80' cy='48' r='4.4'/><path d='M52 192 C36 192 30 178 31 160 C32 142 41 132 40 117 C39 104 22 92 23 75 C24 58 44 52 60 54 C78 56 84 74 80 94 C76 112 69 124 70 146 C71 168 72 192 52 192 Z'/></g></svg>"
#let check-svg(col) = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='M4 12.5l5 5L20 6.5' fill='none' stroke='" + col + "' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'/></svg>"
#let alert-svg(col) = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='M12 3L22 20H2Z' fill='none' stroke='" + col + "' stroke-width='2.2' stroke-linejoin='round'/><path d='M12 9v5' stroke='" + col + "' stroke-width='2.4' stroke-linecap='round'/><circle cx='12' cy='17' r='1.3' fill='" + col + "'/></svg>"
#let phone-svg(col) = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='M6.6 10.8a15.1 15.1 0 006.6 6.6l2.2-2.2a1 1 0 011-.25 11.4 11.4 0 003.6.57 1 1 0 011 1V20a1 1 0 01-1 1A17 17 0 013 4a1 1 0 011-1h3.5a1 1 0 011 1c0 1.25.2 2.45.57 3.57a1 1 0 01-.25 1z' fill='" + col + "'/></svg>"
#let info-svg(col) = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><circle cx='12' cy='12' r='10' fill='none' stroke='" + col + "' stroke-width='2.2'/><path d='M12 11v6' stroke='" + col + "' stroke-width='2.4' stroke-linecap='round'/><circle cx='12' cy='7.5' r='1.4' fill='" + col + "'/></svg>"
#let icon(s, h) = box(height: h, image(bytes(s)))

#let level-col(f) = if f.level == 0 { rgb("#9aa3b2") } else { risk.at(f.level - 1) }
#let level-dark(f) = if f.level == 0 { rgb("#6b7485") } else { riskdark.at(f.level - 1) }
#let dots(f) = box(stack(dir: ltr, spacing: 4pt, ..range(5).map(i => {
  let on = i + 1 == f.level
  box(baseline: 1pt, circle(radius: if on { 5.5pt } else { 4pt },
    fill: if f.level == 0 { rgb("#dfe3ea") } else if on { risk.at(i) } else { risk.at(i).lighten(65%) },
    stroke: if on { 1.5pt + white } else { none }))
})))
#let warn-items = ("luka atau lecet", "kulit melepuh", "bengkak", "kemerahan", "kaki terasa hangat", "keluar cairan atau nanah", "baal atau kesemutan yang baru")

#let bar(right-text) = place(bottom + center, block(width: 210mm, height: 15mm, fill: navy, inset: (x: 12mm),
  align(horizon, grid(columns: (auto, 1fr, auto), column-gutter: 8pt, align: (horizon, left + horizon, right + horizon),
    if d.kontak != "" { icon(phone-svg("#ffffff"), 17pt) } else { [] },
    if d.kontak != "" [#text(fill: white, 9pt)[Ada pertanyaan? Hubungi] \ #text(fill: white, 13pt, weight: "bold")[#d.kontak]] else [#text(fill: white, 10pt)[Ada pertanyaan? Tanyakan ke petugas.]],
    text(fill: white, 10pt)[#right-text #h(6pt) #text(fill: white.transparentize(35%), 9pt)[· Halaman #context counter(page).display() dari #context counter(page).final().first()]]))))

#set document(title: "Hasil skrining kaki - " + c.nama, author: "SmartFoot")
#set page(paper: "a4", margin: (x: 12mm, top: 10mm, bottom: 21mm),
  background: [#image("img/bg.jpg", width: 100%, height: 100%) #place(top + left, rect(width: 100%, height: 100%, fill: gradient.linear(white.transparentize(10%), rgb("#cfe0f7").transparentize(20%), angle: 90deg)))],
  footer: bar(peny))
#set text(font: ("Segoe UI", "Segoe UI Symbol"), size: 11.5pt, fill: ink, lang: "id")

// slanted ribbon, full bleed to the left; the script word is set larger so its x-height matches
#let ribbon(word, script) = move(dx: -12mm, box(width: 140mm, height: 17mm, {
  place(polygon(fill: gradient.linear(navy, blue, angle: 0deg), (0mm, 0mm), (140mm, 0mm), (128mm, 17mm), (0mm, 17mm)))
  place(left + horizon, dx: 12mm, dy: -1mm, [#text(24pt, weight: "bold", fill: white)[#word] #h(3pt) #text(27pt, font: script-font, weight: "bold", fill: rgb("#bcd8ff"))[#script]])
}))
#let name-size = if c.nama.len() > 26 { 12pt } else if c.nama.len() > 18 { 14pt } else { 17pt }
#let for-whom = box(width: 56mm, align(left)[#text(10pt, fill: mute)[untuk] \ #text(name-size, weight: "bold", fill: accent)[#c.sapaan #c.nama]])

// ---------------- page 1 ----------------
#grid(columns: (auto, 1fr), column-gutter: 8pt, align: horizon,
  box(height: 12mm, image(bytes(foot-svg("#1f5fbf")))),
  [#text(16pt, weight: "bold", fill: navy, tracking: 0.5pt)[SMARTFOOT PANORAMA] \ #text(9.5pt, fill: mute)[Skrining kaki untuk penyandang diabetes]])
#v(4pt)
#ribbon[Hasil Skrining][kaki]
#place(top + right, dy: 25mm, for-whom)
#v(3pt)
#text(10pt, fill: mute)[#c.usia tahun · #c.kelamin · #c.tanggal · #c.tempat · Petugas: #c.petugas]
#v(4pt)

#let section(s, title, body) = grid(columns: (auto, 1fr), column-gutter: 8pt,
  box(width: 9mm, height: 9mm, circle(radius: 4.5mm, inset: 0pt, stroke: 1.4pt + blue, fill: white, align(center + horizon, box(height: 5mm, image(bytes(s)))))),
  [#text(13.5pt, weight: "bold", fill: accent)[#title] \ #v(-5pt) #body])
#let rule = [#v(3pt) #line(length: 100%, stroke: hair) #v(3pt)]

#let foot-sec(f) = section(foot-svg(level-col(f).to-hex()), [Kaki #f.sisi])[
  #text(17pt, weight: "bold", fill: level-dark(f))[#f.status] \
  #v(-5pt)
  #dots(f) #h(3pt) #text(9.5pt, fill: mute)[#f.kelas] \
  #text(11pt)[#f.arti]
]

#let foot-pics(a) = block(width: 100%, fill: white, radius: 10pt, stroke: edge, clip: true, inset: 0pt, breakable: false)[
  #block(width: 100%, fill: black, inset: 5pt, spacing: 0pt)[
    #text(10.5pt, weight: "bold", fill: white)[Kaki #a.sisi]
    #v(-6pt)
    #grid(columns: (1fr, 1fr), column-gutter: 4pt,
      box(height: 54mm, width: 100%, clip: true, radius: 4pt, align(center, image(a.scan, height: 100%, fit: "contain"))),
      if a.kontak != "" { box(height: 54mm, width: 100%, clip: true, radius: 4pt, align(center, image(a.kontak, height: 100%, fit: "contain"))) } else { [] })
    #grid(columns: (1fr, 1fr), align(center, text(8pt, fill: white.transparentize(25%))[foto]), align(center, text(8pt, fill: white.transparentize(25%))[#if a.kontak != "" [peta kontak]]))
  ]
  #pad(x: 8pt, y: 6pt, if a.checked { grid(columns: (1fr, auto), row-gutter: 5pt, align: (left, right),
    text(9.5pt, fill: mute)[Bentuk lengkung], text(10.5pt, weight: "bold")[#a.bentuk],
    text(9.5pt, fill: mute)[Panjang kaki], text(10.5pt, weight: "bold")[#a.panjang],
    text(9.5pt, fill: mute)[Lebar kaki], text(10.5pt, weight: "bold")[#a.lebar],
    text(9.5pt, fill: mute)[Indeks lengkung], text(10.5pt, weight: "bold")[#a.ai],
    text(9.5pt, fill: mute)[Sudut Clarke], text(10.5pt, weight: "bold")[#a.clarke]) } else {
    text(10pt, fill: mute)[Ukuran kaki ini belum diperiksa petugas, jadi angkanya belum ditampilkan.] })
]

#let left-panel = block(width: 100%, fill: white, radius: 12pt, inset: 11pt, stroke: edge)[
  #foot-sec(c.feet.at(0))
  #rule
  #foot-sec(c.feet.at(1))
  #rule
  #section(check-svg("#1f5fbf"), [Yang bisa #c.sapaan lakukan])[
    #if c.utama.len() == 0 [
      #text(11pt)[Saran diberikan setelah pemeriksaan kaki lengkap. Tanyakan ke petugas.]
    ] else [
      #for s in c.utama [#text(12pt, weight: "bold")[• #s] \ ]
      #if c.lain.len() > 0 [
        #v(-3pt)
        #text(10pt, fill: mute)[*Juga membantu:* #c.lain.map(s => lower(s.first()) + s.slice(s.first().len())).join(", ").]
      ]
    ]
  ]
  #rule
  #section(alert-svg("#d23c3c"), [Segera ke dokter bila ada])[
    #text(11pt)[#warn-items.join(" · ")] \
    #text(10pt, fill: mute)[Jangan tunggu jadwal kontrol berikutnya.]
  ]
  #if c.catatan.len() > 0 [
    #rule
    #section(info-svg("#1f5fbf"), [Catatan petugas])[#text(11pt)[#c.catatan.join(linebreak())]]
  ]
]

#if c.gambar.len() > 0 {
  grid(columns: (1fr, 70mm), column-gutter: 6pt, left-panel, stack(spacing: 6pt, ..c.gambar.map(foot-pics)))
} else { left-panel }
#v(1fr)
#block(width: 100%, fill: white.transparentize(20%), radius: 8pt, inset: 8pt)[
  #text(10pt)[*Ini hasil skrining awal, bukan diagnosis dokter.* Bawa lembar ini saat #c.sapaan berkonsultasi dengan dokter atau tenaga kesehatan.] \
  #text(7.5pt, fill: mute)[#c.tim_line · dibuat oleh aplikasi SmartFoot #d.versi pada #d.dibuat]
]

// ---------------- page 2: what it means ----------------
#pagebreak()
#set page(footer: bar[#c.sapaan #c.nama · #c.tanggal])
#ribbon[Penjelasan][hasil]
#place(top + right, dy: 1mm, for-whom)
#v(5pt)

#if c.gambar.len() > 0 [
  #let big(a) = block(width: 100%, fill: black, radius: 12pt, inset: 6pt, clip: true)[
    #text(12pt, weight: "bold", fill: white)[Kaki #a.sisi]
    #v(-6pt)
    #grid(columns: (1fr, 1fr), column-gutter: 4pt,
      box(height: 112mm, width: 100%, align(center, image(a.scan, height: 100%, fit: "contain"))),
      if a.kontak != "" { box(height: 112mm, width: 100%, align(center, image(a.kontak, height: 100%, fit: "contain"))) } else { [] })
    #grid(columns: (1fr, 1fr), align(center, text(9pt, fill: white.transparentize(25%))[foto]), align(center, text(9pt, fill: white.transparentize(25%))[#if a.kontak != "" [peta kontak]]))
  ]
  #grid(columns: (1fr, 1fr), column-gutter: 6pt, ..c.gambar.map(big))
  #v(4pt)
  #text(10.5pt)[*Foto* = telapak kaki #c.sapaan dari bawah. *Peta kontak* = bagian telapak yang menyentuh kaca: #box(fill: black, inset: (x: 4pt, y: 1pt), radius: 3pt, text(fill: rgb("#3dff8a"), weight: "bold")[hijau terang]) menekan kuat, hijau redup menekan ringan. Jari kaki di atas, tumit di bawah.]
  #v(5pt)
]

#let explain(title, body) = [#text(11pt, weight: "bold", fill: accent)[#title] #h(4pt) #text(10.5pt)[#body] \ ]
#grid(columns: (1fr, 1fr), column-gutter: 8pt,
  block(width: 100%, fill: white, radius: 12pt, inset: 11pt, stroke: edge)[
    #text(13.5pt, weight: "bold", fill: navy)[Arti angka]
    #v(-2pt)
    #set par(leading: 0.5em, spacing: 0.7em)
    #explain[Bentuk lengkung][Lengkung telapak bagian dalam: datar, normal, atau tinggi.]
    #explain[Panjang kaki][Dari ujung tumit sampai ujung jari terpanjang.]
    #explain[Lebar kaki][Bagian telapak yang paling lebar.]
    #explain[Indeks lengkung][Seberapa banyak bagian tengah telapak menyentuh kaca. Makin besar angkanya, makin banyak yang menyentuh.]
    #explain[Sudut Clarke][Sudut pada lengkung bagian dalam telapak, dipakai untuk menentukan bentuk lengkung.]
  ],
  block(width: 100%, fill: white, radius: 12pt, inset: 11pt, stroke: edge)[
    #text(13.5pt, weight: "bold", fill: navy)[Arti status kaki]
    #v(-2pt)
    #text(10pt, fill: mute)[Petugas memeriksa kulit, kuku, rasa raba, aliran darah, dan bentuk kaki.]
    #v(2pt)
    #let rows = (("Kondisi baik", "risiko sangat rendah"), ("Perlu dijaga", "risiko rendah"), ("Perlu perhatian lebih", "risiko sedang"), ("Perlu penanganan", "risiko tinggi"), ("Segera ke tenaga kesehatan", "risiko urgen"))
    #for (i, r) in rows.enumerate() [
      #grid(columns: (auto, 1fr), column-gutter: 7pt, align: horizon,
        box(circle(radius: 5pt, fill: risk.at(i))),
        [#text(11pt, weight: "bold", fill: riskdark.at(i))[#r.at(0)] #h(3pt) #text(9.5pt, fill: mute)[#r.at(1)]])
      #v(-4pt)
    ]
  ])

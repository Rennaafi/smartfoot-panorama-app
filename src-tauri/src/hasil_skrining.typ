// The patient's result sheet ("Hasil skrining kaki"), two A4 pages, filled from data.json by pdf.rs.
// Layout = the mockup agreed on 25 Nov 2026 (mockups/hasil_skrining/). Text is Bahasa Indonesia.
#let d = json("data.json")
#let c = d.content
#let blue = rgb("#1f5fbf")
#let blue2 = rgb("#e8f0fb")
#let ink = rgb("#14213d")
#let mute = rgb("#6b7a90")
#let line-c = rgb("#d8e0ec")
#let warn = rgb("#d98a00")

#set document(title: "Hasil skrining kaki - " + c.nama, author: "SmartFoot")
#set page(paper: "a4", margin: (x: 16mm, top: 14mm, bottom: 16mm),
  footer: context [
    #set text(9pt, fill: mute)
    #line(length: 100%, stroke: 0.5pt + line-c)
    #v(-4pt)
    Dibuat oleh aplikasi SmartFoot #d.versi pada #d.dibuat #h(1fr) Halaman #counter(page).display() dari #counter(page).final().first()
  ])
#set text(font: ("Segoe UI", "Segoe UI Symbol"), size: 12pt, fill: ink, lang: "id")
#set par(leading: 0.6em)

#let logo = box(height: 15mm, image(bytes("<svg xmlns='http://www.w3.org/2000/svg' viewBox='14 18 76 178'><g fill='#1f5fbf'><ellipse cx='31' cy='39' rx='9.5' ry='12' transform='rotate(-8 31 39)'/><circle cx='48' cy='29' r='6.2'/><circle cx='61' cy='31' r='5.6'/><circle cx='72' cy='38' r='5'/><circle cx='80' cy='48' r='4.4'/><path d='M52 192 C36 192 30 178 31 160 C32 142 41 132 40 117 C39 104 22 92 23 75 C24 58 44 52 60 54 C78 56 84 74 80 94 C76 112 69 124 70 146 C71 168 72 192 52 192 Z'/></g></svg>")))

#let header(title) = {
  let side = ()
  if d.penyelenggara != "" { side.push[Penyelenggara: #d.penyelenggara] }
  if d.kontak != "" { side.push[Kontak: #d.kontak] }
  grid(columns: (auto, 1fr, auto), column-gutter: 10pt, align: horizon,
    logo,
    [#text(10pt, fill: mute)[SmartFoot Panorama] \ #text(22pt, weight: "bold")[#title]],
    align(right, box(width: 70mm, align(right, text(9.5pt, fill: mute, side.join(linebreak()))))))
  v(4pt)
  line(length: 100%, stroke: 1.5pt + blue)
}

#let chip(body, fill) = box(fill: rgb(fill), inset: (x: 7pt, y: 3pt), radius: 4pt, text(weight: "bold", body))

#let foot(f) = block(width: 100%, height: 52mm, stroke: 0.8pt + line-c, radius: 6pt, inset: 12pt)[
  #text(15pt, weight: "bold", fill: blue)[Kaki #f.sisi]
  #v(6pt)
  #text(10pt, fill: mute)[Bentuk lengkung telapak kaki] \
  #if f.arch_ok [#text(16pt, weight: "bold")[#f.arch]] else [#text(weight: "bold", fill: warn)[#f.arch]] \
  #text(10pt, fill: mute)[#f.arch_note]
  #v(8pt)
  #text(10pt, fill: mute)[Risiko kaki (pemeriksaan INLOW)] \
  #chip(f.risk, f.risk_fill)
]

// ---------------- page 1 ----------------
#header[Hasil skrining kaki]
#v(8pt)

#block(width: 100%, fill: blue2, radius: 6pt, inset: 12pt)[
  #grid(columns: (auto, 1fr, auto, 1fr), column-gutter: 8pt, row-gutter: 7pt,
    text(fill: mute)[Nama], [*#c.nama*], text(fill: mute)[Tanggal], c.tanggal,
    text(fill: mute)[Usia], [#c.usia tahun], text(fill: mute)[Tempat], c.tempat,
    text(fill: mute)[Jenis kelamin], c.kelamin, text(fill: mute)[Petugas], c.petugas,
  )
]
#v(10pt)

#grid(columns: (1fr, 1fr), column-gutter: 10pt, rows: auto, ..c.feet.map(foot))
#v(10pt)

#block(width: 100%, stroke: (left: 4pt + blue), inset: (left: 12pt, y: 4pt))[
  #text(15pt, weight: "bold")[Saran]
  #v(2pt)
  #text(10pt, fill: mute)[#c.saran_note]
  #if c.saran.len() > 0 {
    v(4pt)
    set list(spacing: 7pt, marker: text(fill: blue)[•])
    list(..c.saran)
  }
]

#if c.catatan.len() > 0 [
  #v(10pt)
  #text(15pt, weight: "bold")[Catatan petugas]
  #v(2pt)
  #for n in c.catatan [#n \ ]
]
#v(1fr)

#block(width: 100%, fill: rgb("#f5f8fc"), radius: 6pt, inset: 11pt)[
  #set text(10.5pt)
  *Apa arti hasil ini?* Bentuk lengkung dilihat dari gambar telapak kaki: lengkung bisa
  *datar*, *normal*, atau *tinggi*. Risiko kaki berasal dari pemeriksaan INLOW oleh petugas (kulit, kuku,
  rasa raba, aliran darah, dan bentuk kaki), dari *sangat rendah* sampai *ada luka*.
  #v(4pt)
  *Ini hasil skrining, bukan diagnosis.* Bawa lembar ini saat berkonsultasi dengan tenaga kesehatan.
]

// ---------------- page 2 ----------------
#if c.gambar.len() > 0 [
  #pagebreak()
  #header[Gambar kaki]
  #v(6pt)
  #text(10.5pt, fill: mute)[*Pindaian* = gambar telapak kaki dari bawah, diambil dengan _scanner_.
  *Peta kontak* = bagian telapak yang menyentuh kaca: #text(fill: rgb("#1f9d6b"))[*hijau terang*] menekan kuat,
  hijau redup menekan ringan.]
  #v(6pt)

  #let pic(f) = box(stroke: 0.6pt + line-c, radius: 4pt, clip: true, image(f, height: 100mm))
  #for g in c.gambar [
    #grid(columns: (auto, 1fr, 1fr), column-gutter: 8pt, align: (left + horizon, center, center),
      rotate(-90deg, reflow: true, text(14pt, weight: "bold", fill: blue)[Kaki #g.sisi]),
      [#pic(g.scan) \ #text(10pt, fill: mute)[Pindaian]],
      if g.kontak != "" [#pic(g.kontak) \ #text(10pt, fill: mute)[Peta kontak]] else [])
    #v(4pt)
  ]
]

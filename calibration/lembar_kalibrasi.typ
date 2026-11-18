// SmartFoot calibration sheet (A4). Build: typst compile lembar_kalibrasi.typ
//
// Geometry (all mm, from the top-left corner of the paper; the software relies on these numbers):
//   dots      : 5.0 mm diameter, pitch 20.0 mm, 10 columns x 14 rows
//               centres x = 15 + 20*i (i = 0..9), y = 18.5 + 20*j (j = 0..13)
//   markers   : the dots at (i,j) = (0,0), (9,0), (0,13) are 8.0 mm instead of 5.0 mm
//               (three corners, so the software knows which way up the sheet lies)
//   line A    : vertical,   x = 185,   y = 23.5 .. 263.5   (240 mm, along the scan)
//   line B    : horizontal, y = 268.5, x = 15 .. 195       (180 mm, across the scan)
// Print at 100 % ("Actual size"), then measure A and B with a steel ruler and write the results
// on the sheet: the software uses the MEASURED lengths, so printer scaling cancels out.

#set page(width: 210mm, height: 297mm, margin: 0pt)
#set text(font: ("Segoe UI", "Arial"), size: 7pt)

#let pitch = 20mm
#let x0 = 15mm
#let y0 = 18.5mm
#let cols = 10
#let rows = 14
#let markers = ((0, 0), (9, 0), (0, 13))

#let dot(x, y, d) = place(top + left, dx: x - d / 2, dy: y - d / 2, circle(radius: d / 2, fill: black, stroke: none))
#let seg(a, b) = place(top + left, line(start: a, end: b, stroke: 0.2mm + black))

// dots
#for j in range(rows) {
  for i in range(cols) {
    let d = if markers.contains((i, j)) { 8mm } else { 5mm }
    dot(x0 + i * pitch, y0 + j * pitch, d)
  }
}

// line A (vertical, 240 mm) with end ticks
#let ax = 185mm
#seg((ax, 23.5mm), (ax, 263.5mm))
#seg((ax - 3mm, 23.5mm), (ax + 3mm, 23.5mm))
#seg((ax - 3mm, 263.5mm), (ax + 3mm, 263.5mm))
#place(top + left, dx: ax + 1.2mm, dy: 140mm, rotate(90deg, reflow: true)[*A* = 240 mm (ujung garis ke ujung garis)])

// line B (horizontal, 180 mm) with end ticks
#let by = 268.5mm
#seg((15mm, by), (195mm, by))
#seg((15mm, by - 3mm), (15mm, by + 3mm))
#seg((195mm, by - 3mm), (195mm, by + 3mm))
#place(top + left, dx: 80mm, dy: by + 0.8mm)[*B* = 180 mm (ujung garis ke ujung garis)]

// header (above the first row of dots)
#place(top + left, dx: 30mm, dy: 4mm)[
  #text(size: 9pt, weight: "bold")[SmartFoot · Lembar kalibrasi skala]
  #h(4mm) Cetak *100 % / Actual size* (jangan "Fit to page"). Letakkan rata di atas akrilik, tulisan menghadap ke kaca.
]

// footer (below the last row of dots): measured lengths
#place(top + left, dx: 15mm, dy: 284mm)[
  Hasil ukur dengan penggaris baja (di atas meja): #h(2mm)
  *A* = #box(width: 22mm, stroke: (bottom: 0.3pt))[] mm #h(4mm)
  *B* = #box(width: 22mm, stroke: (bottom: 0.3pt))[] mm #h(4mm)
  Tanggal: #box(width: 22mm, stroke: (bottom: 0.3pt))[] #h(4mm)
  Oleh: #box(width: 20mm, stroke: (bottom: 0.3pt))[]
]
#place(top + left, dx: 15mm, dy: 290mm, text(size: 6pt, fill: luma(90))[
  Titik 5 mm, jarak 20 mm (10 × 14); 3 titik 8 mm di sudut = penanda arah. Jangan dilipat; ganti lembar bila kusut atau basah.
])

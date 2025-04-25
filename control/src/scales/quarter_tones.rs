// ALLOW: Any of the invariants may be constructed using `try_from_u8`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, defmt::Format)]
pub enum QuarterTone {
    CMinus1 = 0,
    CQSharpMinus1,
    CSharpMinus1,
    C3QSharpMinus1,
    DMinus1,
    DQSharpMinus1,
    DSharpMinus1,
    D3QSharpMinus1,
    EMinus1,
    EQSharpMinus1,
    FMinus1,
    FQSharpMinus1,
    FSharpMinus1,
    F3QSharpMinus1,
    GMinus1,
    GQSharpMinus1,
    GSharpMinus1,
    G3QSharpMinus1,
    AMinus1,
    AQSharpMinus1,
    ASharpMinus1,
    A3QSharpMinus1,
    BMinus1,
    BQSharpMinus1,
    C0,
    CQSharp0,
    CSharp0,
    C3QSharp0,
    D0,
    DQSharp0,
    DSharp0,
    D3QSharp0,
    E0,
    EQSharp0,
    F0,
    FQSharp0,
    FSharp0,
    F3QSharp0,
    G0,
    GQSharp0,
    GSharp0,
    G3QSharp0,
    A0,
    AQSharp0,
    ASharp0,
    A3QSharp0,
    B0,
    BQSharp0,
    C1,
    CQSharp1,
    CSharp1,
    C3QSharp1,
    D1,
    DQSharp1,
    DSharp1,
    D3QSharp1,
    E1,
    EQSharp1,
    F1,
    FQSharp1,
    FSharp1,
    F3QSharp1,
    G1,
    GQSharp1,
    GSharp1,
    G3QSharp1,
    A1,
    AQSharp1,
    ASharp1,
    A3QSharp1,
    B1,
    BQSharp1,
    C2,
    CQSharp2,
    CSharp2,
    C3QSharp2,
    D2,
    DQSharp2,
    DSharp2,
    D3QSharp2,
    E2,
    EQSharp2,
    F2,
    FQSharp2,
    FSharp2,
    F3QSharp2,
    G2,
    GQSharp2,
    GSharp2,
    G3QSharp2,
    A2,
    AQSharp2,
    ASharp2,
    A3QSharp2,
    B2,
    BQSharp2,
    C3,
    CQSharp3,
    CSharp3,
    C3QSharp3,
    D3,
    DQSharp3,
    DSharp3,
    D3QSharp3,
    E3,
    EQSharp3,
    F3,
    FQSharp3,
    FSharp3,
    F3QSharp3,
    G3,
    GQSharp3,
    GSharp3,
    G3QSharp3,
    A3,
    AQSharp3,
    ASharp3,
    A3QSharp3,
    B3,
    BQSharp3,
    C4,
    CQSharp4,
    CSharp4,
    C3QSharp4,
    D4,
    DQSharp4,
    DSharp4,
    D3QSharp4,
    E4,
    EQSharp4,
    F4,
    FQSharp4,
    FSharp4,
    F3QSharp4,
    G4,
    GQSharp4,
    GSharp4,
    G3QSharp4,
    A4,
    AQSharp4,
    ASharp4,
    A3QSharp4,
    B4,
    BQSharp4,
    C5,
    CQSharp5,
    CSharp5,
    C3QSharp5,
    D5,
    DQSharp5,
    DSharp5,
    D3QSharp5,
    E5,
    EQSharp5,
    F5,
    FQSharp5,
    FSharp5,
    F3QSharp5,
    G5,
    GQSharp5,
    GSharp5,
    G3QSharp5,
    A5,
    AQSharp5,
    ASharp5,
    A3QSharp5,
    B5,
    BQSharp5,
    C6,
    CQSharp6,
    CSharp6,
    C3QSharp6,
    D6,
    DQSharp6,
    DSharp6,
    D3QSharp6,
    E6,
    EQSharp6,
    F6,
    FQSharp6,
    FSharp6,
    F3QSharp6,
    G6,
    GQSharp6,
    GSharp6,
    G3QSharp6,
    A6,
    AQSharp6,
    ASharp6,
    A3QSharp6,
    B6,
    BQSharp6,
    C7,
    CQSharp7,
    CSharp7,
    C3QSharp7,
    D7,
    DQSharp7,
    DSharp7,
    D3QSharp7,
    E7,
    EQSharp7,
    F7,
    FQSharp7,
    FSharp7,
    F3QSharp7,
    G7,
    GQSharp7,
    GSharp7,
    G3QSharp7,
    A7,
    AQSharp7,
    ASharp7,
    A3QSharp7,
    B7,
    BQSharp7,
    C8,
    CQSharp8,
    CSharp8,
    C3QSharp8,
    D8,
    DQSharp8,
    DSharp8,
    D3QSharp8,
    E8,
    EQSharp8,
    F8,
    FQSharp8,
    FSharp8,
    F3QSharp8,
    G8,
    GQSharp8,
    GSharp8,
    G3QSharp8,
    A8,
    AQSharp8,
    ASharp8,
    A3QSharp8,
    B8,
    BQSharp8,
    C9,
}

// NOTE: Even after accounting for phase offset (which helped a lot), I did
// not figure out how to adjust the synthesis algorithm to be in tune, so
// I tuned everything manually.
// The tuning process was simply going over all the semitones with resonance
// set at 11 o'clock and cutoff at 12, recording the output into fmit and
// its "error history" graph. And then compensating the frequency with a
// coefficient to make it in tune.
// The results were not very stable, they swayed with the trigger interval
// a lot. I settled for tolerance of 2 cents in both directions, with most
// notes being within 1 cent from the center of the note. But depending on
// the resonance, cutoff and trigger frequency, the results may vary.
// NOTE: Coefficients on quarter tones are simply copying the one from the
// previous semitone tone. It did not feel important enough to tune all the
// quarter notes.
#[allow(clippy::excessive_precision)]
const FREQUENCIES: [f32; 241] = [
    8.17579892,
    8.41536811,
    8.66195722,
    8.91577194,
    9.17702400,
    9.44593133,
    9.72271824,
    10.00761563,
    10.30086115,
    10.60269942,
    10.91338223,
    11.23316874,
    11.56232571,
    11.90112771,
    12.24985737,
    12.60880559,
    12.97827180,
    13.35856419,
    13.75000000,
    14.15290575,
    14.56761755,
    14.99448132,
    15.43385316,
    15.88609958,
    16.35159783 * 1.00098570120696, // C0, this is as low as the module will go
    16.83073622 * 1.00098570120696, // CQSharp0
    17.32391444 * 0.99972757376226, // CSharp0
    17.83154388 * 0.99972757376226, // C3QSharp0
    18.35404799 * 1.00070532025632, // D0
    18.89186265 * 1.00070532025632, // DQSharp0
    19.44543648 * 0.9997638847802399, // DSharp0
    20.01523126 * 0.9997638847802399, // D3QSharp0
    20.60172231 * 0.9996888564546801, // E0
    21.20539885 * 0.9996888564546801, // EQSharp0
    21.82676446 * 1.00038152677176, // F0
    22.46633748 * 1.00038152677176, // FQSharp0
    23.12465142 * 1.0000947163509384, // FSharp0
    23.80225543 * 1.0000947163509384, // F3QSharp0
    24.49971475 * 1.00070978493096, // G0
    25.21761119 * 1.00070978493096, // GQSharp0
    25.95654360 * 1.000528516550472, // GSharp0
    26.71712838 * 1.000528516550472, // G3QSharp0
    27.50000000 * 1.00110790611844, // A0
    28.30581151 * 1.00110790611844, // AQSharp0
    29.13523509 * 0.9999502848987786, // ASharp0
    29.98896265 * 0.9999502848987786, // A3QSharp0
    30.86770633 * 1.0005286,        // B0
    31.77219916 * 1.0005286,        // BQSharp0
    32.70319566 * 1.0005493974938047, // C1
    33.66147244 * 1.0005493974938047, // CQSharp1
    34.64782887 * 1.00052076697272, // CSharp1
    35.66308775 * 1.00052076697272, // C3QSharp1
    36.70809599 * 1.0006791600127036, // D1
    37.78372531 * 1.0006791600127036, // DQSharp1
    38.89087297 * 1.00039019091291, // DSharp1
    40.03046253 * 1.00039019091291, // D3QSharp1
    41.20344461 * 1.000607,         // E1
    42.41079770 * 1.000607,         // EQSharp1
    43.65352893 * 0.9999452,        // F1
    44.93267496 * 0.9999452,        // FQSharp1
    46.24930284 * 1.00023408420264, // FSharp1
    47.60451086 * 1.00023408420264, // F3QSharp1
    48.99942950 * 1.000454653667385, // G1
    50.43522238 * 1.000454653667385, // GQSharp1
    51.91308720 * 1.0008340163630256, // GSharp1
    53.43425676 * 1.0008340163630256, // G3QSharp1
    55.00000000 * 1.0008316,        // A1
    56.61162302 * 1.0008316,        // AQSharp1
    58.27047019 * 1.00038387740649, // ASharp1
    59.97792530 * 1.00038387740649, // A3QSharp1
    61.73541266 * 1.0008667668686402, // B1
    63.54439833 * 1.0008667668686402, // BQSharp1
    65.40639133 * 1.0010782585389577, // C2
    67.32294488 * 1.0010782585389577, // CQSharp2
    69.29565774 * 1.0007932817474368, // CSharp2
    71.32617551 * 1.0007932817474368, // C3QSharp2
    73.41619198 * 1.0005529138202527, // D2
    75.56745061 * 1.0005529138202527, // DQSharp2
    77.78174593 * 1.0007307607378373, // DSharp2
    80.06092506 * 1.0007307607378373, // D3QSharp2
    82.40688923 * 1.00138534908908, // E2
    84.82159540 * 1.00138534908908, // EQSharp2
    87.30705786 * 1.0020989729370104, // F2
    89.86534993 * 1.0020989729370104, // FQSharp2
    92.49860568 * 1.00141322464008, // FSharp2
    95.20902171 * 1.00141322464008, // F3QSharp2
    97.99885900 * 1.00185363675278, // G2
    100.87044475 * 1.00185363675278, // GQSharp2
    103.82617439 * 1.0013696849307068, // GSharp2
    106.86851353 * 1.0013696849307068, // G3QSharp2
    110.00000000 * 1.0022523647206503, // A2
    113.22324603 * 1.0022523647206503, // AQSharp2
    116.54094038 * 1.0014056883970501, // ASharp2
    119.95585059 * 1.0014056883970501, // A3QSharp2
    123.47082531 * 1.0015278386751394, // B2
    127.08879666 * 1.0015278386751394, // BQSharp2
    130.81278265 * 1.002122260512798, // C3
    134.64588976 * 1.002122260512798, // CQSharp3
    138.59131549 * 1.00212147017216, // CSharp3
    142.65235101 * 1.00212147017216, // C3QSharp3
    146.83238396 * 1.0017636428852505, // D3
    151.13490122 * 1.0017636428852505, // DQSharp3
    155.56349186 * 1.00205323603317, // DSharp3
    160.12185011 * 1.00205323603317, // D3QSharp3
    164.81377846 * 1.0023451996936725, // E3
    169.64319079 * 1.0023451996936725, // EQSharp3
    174.61411572 * 1.0021980979634597, // F3
    179.73069986 * 1.0021980979634597, // FQSharp3
    184.99721136 * 1.002041945583505, // FSharp3
    190.41804342 * 1.002041945583505, // F3QSharp3
    195.99771799 * 1.0022727600077599, // G3
    201.74088951 * 1.0022727600077599, // GQSharp3
    207.65234879 * 1.0029330634428002, // GSharp3
    213.73702705 * 1.0029330634428002, // G3QSharp3
    220.00000000 * 1.0027094170420725, // A3
    226.44649206 * 1.0027094170420725, // AQSharp3
    233.08188076 * 1.002821702336268, // ASharp3
    239.91170119 * 1.002821702336268, // A3QSharp3
    246.94165063 * 1.00269525908223, // B3
    254.17759331 * 1.00269525908223, // BQSharp3
    261.62556530 * 1.00330641631878, // C4
    269.29177953 * 1.00330641631878, // CQSharp4
    277.18263098 * 1.0038446,       // CSharp4
    285.30470202 * 1.0038446,       // C3QSharp4
    293.66476792 * 1.0031136173616602, // D4
    302.26980244 * 1.0031136173616602, // DQSharp4
    311.12698372 * 1.0043372539930677, // DSharp4
    320.24370023 * 1.0043372539930677, // D3QSharp4
    329.62755691 * 1.0033220153875595, // E4
    339.28638159 * 1.0033220153875595, // EQSharp4
    349.22823143 * 1.0045741,       // F4
    359.46139971 * 1.0045741,       // FQSharp4
    369.99442271 * 1.0036841808277472, // FSharp4
    380.83608684 * 1.0036841808277472, // F3QSharp4
    391.99543598 * 1.0042436723093036, // G4
    403.48177901 * 1.0042436723093036, // GQSharp4
    415.30469758 * 1.003272282984959, // GSharp4
    427.47405411 * 1.003272282984959, // G3QSharp4
    440.00000000 * 1.0061310285378038, // A4
    452.89298412 * 1.0061310285378038, // AQSharp4
    466.16376152 * 1.0063483,       // ASharp4
    479.82340237 * 1.0063483,       // A3QSharp4
    493.88330126 * 1.0070432335702602, // B4
    508.35518662 * 1.0070432335702602, // BQSharp4
    523.25113060 * 1.006731754133336, // C5
    538.58355905 * 1.006731754133336, // CQSharp5
    554.36526195 * 1.005797022112212, // CSharp5
    570.60940405 * 1.005797022112212, // C3QSharp5
    587.32953583 * 1.006492912400787, // D5
    604.53960488 * 1.006492912400787, // DQSharp5
    622.25396744 * 1.0103774589732002, // DSharp5
    640.48740045 * 1.0103774589732002, // D3QSharp5
    659.25511383 * 1.0085444,       // E5
    678.57276318 * 1.0085444,       // EQSharp5
    698.45646287 * 1.0091161380502915, // F5
    718.92279943 * 1.0091161380502915, // FQSharp5
    739.98884542 * 1.008828110693889, // FSharp5
    761.67217369 * 1.008828110693889, // F3QSharp5
    783.99087196 * 1.009152685794,  // G5
    806.96355802 * 1.009152685794,  // GQSharp5
    830.60939516 * 1.00989702504376, // GSharp5
    854.94810822 * 1.00989702504376, // G3QSharp5
    880.00000000 * 1.008148589982,  // A5
    905.78596825 * 1.008148589982,  // AQSharp5
    932.32752304 * 1.0126582,       // ASharp5
    959.64680475 * 1.0126582,       // A3QSharp5
    987.76660251 * 1.0066077466271999, // B5
    1016.71037325 * 1.0066077466271999, // BQSharp5
    1046.50226120 * 1.0133780867191802, // C6
    1077.16711811 * 1.0133780867191802, // CQSharp6
    1108.73052391 * 1.01220765410758, // CSharp6
    1141.21880809 * 1.01220765410758, // C3QSharp6
    1174.65907167 * 1.0147776780581999, // D6
    1209.07920976 * 1.0147776780581999, // DQSharp6
    1244.50793489 * 1.00911155185323, // DSharp6
    1280.97480090 * 1.00911155185323, // D3QSharp6
    1318.51022765 * 1.0116903,      // E6
    1357.14552636 * 1.0116903,      // EQSharp6
    1396.91292573 * 1.0229461344308, // F6
    1437.84559885 * 1.0229461344308, // FQSharp6
    1479.97769085 * 1.0134392587296799, // FSharp6
    1523.34434737 * 1.0134392587296799, // F3QSharp6
    1567.98174393 * 1.0157181,      // G6
    1613.92711604 * 1.0157181,      // GQSharp6
    1661.21879032 * 1.0234566,      // GSharp6
    1709.89621643 * 1.0234566,      // G3QSharp6
    1760.00000000 * 1.0204226,      // A6
    1811.57193649 * 1.0204226,      // AQSharp6
    1864.65504607 * 1.0220162702258702, // ASharp6
    1919.29360949 * 1.0220162702258702, // A3QSharp6
    1975.53320502 * 1.0145453,      // B6
    2033.42074650 * 1.0145453,      // BQSharp6
    2093.00452240 * 1.03256687261268, // C7
    2154.33423622 * 1.03256687261268, // CQSharp7
    2217.46104781 * 1.0201279,      // CSharp7
    2282.43761619 * 1.0201279,      // C3QSharp7
    2349.31814334 * 1.008702,       // D7
    2418.15841953 * 1.008702,       // DQSharp7
    2489.01586978 * 1.0028923,      // DSharp7
    2561.94960180 * 1.0028923,      // D3QSharp7
    2637.02045530 * 0.9959648,      // E7
    2714.29105272 * 0.9959648,      // EQSharp7
    2793.82585146 * 1.0376596192129002, // F7
    2875.69119770 * 1.0376596192129002, // FQSharp7
    2959.95538169 * 1.0210121,      // FSharp7
    3046.68869474 * 1.0210121,      // F3QSharp7
    3135.96348785 * 1.013374,       // G7
    3227.85423208 * 1.013374,       // GQSharp7
    3322.43758064 * 1.0174797,      // GSharp7
    3419.79243286 * 1.0174797,      // G3QSharp7
    3520.00000000 * 1.0198333,      // A7
    3623.14387299 * 1.0198333,      // AQSharp7
    3729.31009214 * 1.0322791988075999, // ASharp7
    3838.58721898 * 1.0322791988075999, // A3QSharp7
    3951.06641005 * 1.0263338,      // B7, this is as high as the module will go
    4066.84149299,
    4186.00904481,
    4308.66847243,
    4434.92209563,
    4564.87523237,
    4698.63628668,
    4836.31683905,
    4978.03173955,
    5123.89920360,
    5274.04091061,
    5428.58210544,
    5587.65170293,
    5751.38239541,
    5919.91076339,
    6093.37738948,
    6271.92697571,
    6455.70846416,
    6644.87516128,
    6839.58486572,
    7040.00000000,
    7246.28774597,
    7458.62018429,
    7677.17443796,
    7902.13282010,
    8133.68298598,
    8372.01808962,
];

impl QuarterTone {
    const HIGHEST_NOTE: Self = Self::BQSharp8;

    pub fn voct(self) -> f32 {
        self as u8 as f32 / 24.0
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn try_from_u8(index: u8) -> Option<QuarterTone> {
        if index <= Self::HIGHEST_NOTE.index() {
            Some(unsafe { core::mem::transmute::<u8, Self>(index) })
        } else {
            None
        }
    }

    pub fn frequency(self) -> f32 {
        FREQUENCIES[self as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_voct_from_tone() {
        assert_eq!(QuarterTone::CMinus1.voct(), 0.0);
        assert_eq!(QuarterTone::C0.voct(), 1.0);
        assert_eq!(QuarterTone::CSharp0.voct(), 1.0 + 1.0 / 12.0);
    }
}

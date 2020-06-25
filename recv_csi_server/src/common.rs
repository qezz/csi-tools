use std::fs::File;
use chrono::prelude::Utc;

use crate::types::Sample;

pub fn save_collected(csi: &Vec<Sample>) {
    let date = match csi.first() {
        Some(s) => s.date,
        None => Utc::now(),
    };
    let fname = format!("csi_data_{}.csv", date);

    let output = File::create(&fname).unwrap();
    let mut wtr = csv::Writer::from_writer(output);

    for r in csi {
        wtr.write_record(
            [
                vec![
                    format!("{}", r.date),
                    format!("{}", r.x),
                    format!("{}", r.y),
                ],
                r.csi[0][0].iter().map(ToString::to_string).collect(),
                r.csi[0][1].iter().map(ToString::to_string).collect(),
                r.csi[1][0].iter().map(ToString::to_string).collect(),
                r.csi[1][1].iter().map(ToString::to_string).collect(),
            ].concat()
        ).unwrap();
    }

    wtr.flush()
        .unwrap();
}


// use crate::types::CSIData;

// pub fn save_collected_(csi: &CSIData) {
//     let c = csi.c.as_ref().unwrap();
//     let fname = format!("csi_data.csv");

//     let output = File::create(&fname)
//         .unwrap();
//     let mut wtr = csv::Writer::from_writer(output);

//     for r in &csi.inner {
//         wtr.write_record(
//             [
//                 vec![
//                     format!("{}", 1),
//                     format!("{}", 0),
//                     format!("{}", 0),
//                 ],
//                 r[0][0].iter().map(ToString::to_string).collect(),
//                 r[0][1].iter().map(ToString::to_string).collect(),
//                 r[1][0].iter().map(ToString::to_string).collect(),
//                 r[1][1].iter().map(ToString::to_string).collect(),
//             ].concat()
//         ).unwrap();
//     }

//     wtr.flush()
//         .unwrap();
// }

use std::error::Error;
use geo_types::{LineString, MultiLineString, Polygon, Geometry};
use geos::{Geom, Geometry as GeosGeometry, GResult};
use std::convert::TryFrom;


/*
impl<'a, 'b> TryFrom<&'a Geometry<f64>> for GeosGeometry<'b> {
    type Error = &'static str;

    fn try_from(other: &'a Geometry<f64>) -> Result<GeosGeometry<'b>, Self::Error> {
        match other {
            Geometry::LineString(line) => geos::Geometry::try_from(line),
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            Geometry::MultiLineString(mls) => {
                geos::Geometry::create_multiline_string(mls.0
                    .clone()
                    .iter()
                    .map(|line| {
                        geos::Geometry::try_from(line)
                            .unwrap_or(geos::Geometry::create_empty_line_string().unwrap())
                    })
                    .collect())
            },
            _ => Err(geos::Error::InvalidGeometry("Wrong type of geometry".into()))
        }
    }

}
 */

pub trait LineClip {
    fn clipwith(&self, clipobj: &Self) -> Result<MultiLineString<f64>, Box<dyn Error>>;
}

impl LineClip for Geometry<f64>
{
    fn clipwith(&self, clipobj: &Self) -> Result<MultiLineString<f64>, Box<dyn Error>> {
        let geo_self: geos::Geometry = match self {
            Geometry::LineString(line) => geos::Geometry::try_from(line),
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            Geometry::MultiLineString(mls) => {
                geos::Geometry::create_multiline_string(mls.0
                    .clone()
                    .iter()
                    .map(|line| {
                        geos::Geometry::try_from(line)
                            .unwrap_or(geos::Geometry::create_empty_line_string().unwrap())
                    })
                    .collect())
            },
            _ => Err(geos::Error::InvalidGeometry("Wrong type of geometry".into()))
        }?;

        let geo_clipping_obj = match clipobj {
            Geometry::LineString(the_line) => {
                geos::Geometry::try_from(
                    Polygon::new(the_line.clone(), vec![]))?
                        .buffer(0.001, 8)
            },
            Geometry::Polygon(poly) => geos::Geometry::try_from(poly),
            _ => Err(geos::Error::InvalidGeometry("Wrong type of geometry".into()))
        }?;
        let clipped_self = geo_self.difference(&geo_clipping_obj)?;
        let gt_out: geo_types::Geometry<f64> = geo_types::Geometry::try_from(clipped_self)?;

        match gt_out {
            geo_types::Geometry::MultiLineString(mls) => Ok(mls),
            geo_types::Geometry::GeometryCollection(gc) => {
                let foo = gc.iter().map(|g| match g {
                    geo_types::Geometry::MultiLineString(mls) => mls.clone(),
                    geo_types::Geometry::LineString(ls) => MultiLineString(vec![ls.clone()]),
                    _ => MultiLineString(vec![])
                }).collect::<Vec<MultiLineString<f64>>>();
                Ok(MultiLineString::new(foo.iter().map(|mls| mls.0.clone()).flatten().collect()))
            },
            _ => Ok(MultiLineString::new(vec![])),
        }


    }
}

impl LineClip for LineString<f64> {
    fn clipwith(&self, clipobj: &Self) -> Result<MultiLineString<f64>, Box<dyn Error>> {
        let geo_self: geos::Geometry = geos::Geometry::try_from(self)?;
        let geo_clipping_obj = geos::Geometry::try_from(Polygon::new(clipobj.clone(), vec![]))?;
        let geo_clipping_obj = geo_clipping_obj.buffer(0.001, 4)?;
        let clipped_self = geo_self.difference(&geo_clipping_obj)?;
        let gt_out: geo_types::Geometry<f64> = geo_types::Geometry::try_from(clipped_self)?;
        match gt_out {
            geo_types::Geometry::MultiLineString(mls) => Ok(mls),
            geo_types::Geometry::GeometryCollection(gc) => {
                let foo = gc.iter().map(|g| match g {
                    geo_types::Geometry::MultiLineString(mls) => mls.clone(),
                    geo_types::Geometry::LineString(ls) => MultiLineString(vec![ls.clone()]),
                    _ => MultiLineString(vec![])
                }).collect::<Vec<MultiLineString<f64>>>();
                Ok(MultiLineString::new(foo.iter().map(|mls| mls.0.clone()).flatten().collect()))
            },
            _ => Ok(MultiLineString::new(vec![])),
        }
    }
}

#[cfg(test)]
mod test {
    use std::f64::consts::PI;
    use geo_types::{coord, Rect, Polygon, LineString};
    use geos::{Geom, Geometry};
    use wkt::TryFromWkt;
    use super::*;
    #[test]
    fn test_clip_joydiv(){
        let front = LineString::try_from_wkt_str("LINESTRING (10.0000000000000000 20.0000000000000000, 10.5949999999999989 19.9999999999999964, 11.3600000000000012 20.0000000000000036, 12.2649999999999988 20.0000000000000000, 13.2799999999999976 20.0000000000000000, 14.3750000000000000 20.0000000000000000, 15.5200000000000014 20.0000000000000000, 16.6850000000000023 20.0000000000000000, 17.8399999999999999 20.0000000000000000, 18.9550000000000018 20.0000000000000000, 20.0000000000000000 20.0000000000000000, 20.9999999999999964 19.9999999999999964, 22.0000000000000036 20.0000000000000036, 23.0000000000000000 20.0000000000000000, 24.0000000000000000 20.0000000000000000, 25.0000000000000000 20.0000000000000000, 26.0000000000000036 20.0000000000000000, 26.9999999999999964 20.0000000000000000, 28.0000000000000000 20.0000000000000000, 29.0000000000000000 20.0000000000000000, 30.0000000000000000 20.0000000000000000, 31.0000000000000000 19.9999999999999964, 32.0000000000000000 20.0000000000000036, 32.9999999999999929 20.0000000000000000, 34.0000000000000000 20.0000000000000000, 35.0000000000000000 20.0000000000000000, 36.0000000000000071 20.0000000000000000, 37.0000000000000000 20.0000000000000000, 38.0000000000000000 20.0000000000000000, 39.0000000000000000 20.0000000000000000, 40.0000000000000000 20.0000000000000000, 40.9999999999999929 20.0216277888341878, 42.0000000000000071 20.0768988047437915, 42.9999999999999929 20.1513945218393324, 43.9999999999999929 20.2306964142313674, 45.0000000000000000 20.3003859560304249, 46.0000000000000071 20.3460446213470476, 47.0000000000000071 20.3532538842917781, 48.0000000000000000 20.3075952189751554, 49.0000000000000000 20.1946500995077152, 50.0000000000000000 20.0000000000000000, 51.0000000000000000 19.7083631000475883, 52.0000000000000071 19.3262020447926623, 53.0000000000000000 18.8717150066656281, 54.0000000000000000 18.3631001580969198, 55.0000000000000000 17.8185556715169575, 56.0000000000000071 17.2562797193561721, 57.0000000000000000 16.6944704740449801, 58.0000000000000000 16.1513261080138015, 59.0000000000000000 15.6450447936930690, 60.0000000000000000 15.1938247035132026, 61.0000000000000000 14.7503522106201235, 62.0000000000000000 14.2694697879448338, 62.9999999999999929 13.7726097575953670, 64.0000000000000142 13.2812044416797708, 65.0000000000000000 12.8166861623060715, 65.9999999999999858 12.4004872415823151, 67.0000000000000000 12.0540400016165421, 67.9999999999999858 11.7987767645167825, 69.0000000000000000 11.6561298523910786, 70.0000000000000000 11.6475315873474710, 70.9999999999999858 11.8485698935421198, 72.0000000000000142 12.2790119503571482, 73.0000000000000000 12.8765589627395389, 74.0000000000000000 13.5789121356362745, 75.0000000000000000 14.3237726739943376, 76.0000000000000000 15.0488417827607091, 77.0000000000000000 15.6918206668823679, 77.9999999999999858 16.1904105313062985, 79.0000000000000000 16.4823125809794817, 80.0000000000000000 16.5052280208489037, 80.9999999999999858 16.2421095618403584, 82.0000000000000142 15.7480926456342640, 83.0000000000000000 15.0691525733095482, 84.0000000000000000 14.2512646459451346, 85.0000000000000000 13.3404041646199527, 86.0000000000000000 12.3825464304129174, 87.0000000000000000 11.4236667444029614, 88.0000000000000000 10.5097404076690015, 89.0000000000000000 9.6867427212899653, 90.0000000000000000 9.0006489863447783, 91.0000000000000000 8.4317167467626088, 92.0000000000000142 7.9212998813852309, 93.0000000000000000 7.4570181936604563, 94.0000000000000142 7.0264914870360995, 95.0000000000000000 6.6173395649599778, 96.0000000000000000 6.2171822308799065, 97.0000000000000000 5.8136392882437011, 97.9999999999999858 5.3943305404991744, 99.0000000000000000 4.9468757910941434, 100.0000000000000000 4.4588948434764255, 101.0000000000000000 3.8922038497180198, 102.0000000000000142 3.2384282833204554, 103.0000000000000142 2.5299019298758316, 104.0000000000000142 1.7989585749762496, 105.0000000000000000 1.0779320042138099, 106.0000000000000000 0.3991560031806133, 107.0000000000000000 -0.2050356425312363, 108.0000000000000000 -0.7023091473296432, 109.0000000000000000 -1.0603307256225021, 110.0000000000000000 -1.2467665918177140, 111.0000000000000000 -1.2344990202901340, 112.0000000000000142 -1.0419939411213055, 113.0000000000000142 -0.7052930522791582, 114.0000000000000000 -0.2604380517316218, 115.0000000000000000 0.2565293625533722, 116.0000000000000000 0.8095674926078954, 116.9999999999999858 1.3626346404640159, 117.9999999999999858 1.8796891081538059, 119.0000000000000000 2.3246891977093327, 120.0000000000000000 2.6615932111626677, 121.0000000000000000 3.0110001923865006, 122.0000000000000000 3.4772861858694402, 123.0000000000000000 4.0000749945674308, 124.0000000000000142 4.5189904214364169, 125.0000000000000000 4.9736562694323450, 126.0000000000000000 5.3036963415111611, 127.0000000000000000 5.4487344406288090, 128.0000000000000000 5.3483943697412339, 129.0000000000000000 4.9422999318043832, 130.0000000000000000 4.1700749297742021, 131.0000000000000000 2.9055885164774411, 132.0000000000000284 1.1413803515302052, 133.0000000000000000 -1.0049200019275326, 134.0000000000000000 -3.4156829807557987, 135.0000000000000000 -5.9732790218146183, 136.0000000000000000 -8.5600785619640227, 137.0000000000000000 -11.0584520380640292, 138.0000000000000000 -13.3507698869746783, 139.0000000000000000 -15.3194025455559846, 140.0000000000000000 -16.8467204506679806, 140.9999999999999716 -17.9180188620546907, 142.0000000000000000 -18.6471146932005603, 143.0000000000000284 -19.1091608615762070, 143.9999999999999716 -19.3793102846522700, 145.0000000000000000 -19.5327158798993814, 146.0000000000000000 -19.6445305647881732, 147.0000000000000000 -19.7899072567892738, 147.9999999999999716 -20.0439988733733223, 149.0000000000000000 -20.4819583320109473, 150.0000000000000000 -21.1789385501727736, 150.9999999999999716 -22.3203276778005240, 152.0000000000000284 -23.9311827726700379, 153.0000000000000284 -25.8461601140031689, 154.0000000000000000 -27.8999159810217456, 155.0000000000000000 -29.9271066529476215, 156.0000000000000284 -31.7623884090026323, 157.0000000000000000 -33.2404175284086207, 157.9999999999999716 -34.1958502903874262, 159.0000000000000000 -34.4633429741608950, 160.0000000000000000 -33.8775518589508735, 160.9999999999999716 -32.1485103488569308, 162.0000000000000284 -29.2865934528895160, 163.0000000000000000 -25.5769698576367759, 163.9999999999999716 -21.3048082496868538, 165.0000000000000000 -16.7552773156279109, 166.0000000000000284 -12.2135457420480815, 166.9999999999999716 -7.9647822155355268, 168.0000000000000000 -4.2941554226783687, 169.0000000000000284 -1.4868340500647816, 170.0000000000000000 0.1720132157171044, 171.0000000000000000 0.6453837490708144, 172.0000000000000000 0.2084510255712032, 173.0000000000000000 -0.9556894896128898, 173.9999999999999716 -2.6639423313126258, 175.0000000000000000 -4.7332120343591635, 176.0000000000000284 -6.9804031335836694, 176.9999999999999716 -9.2224201638172936, 178.0000000000000000 -11.2761676598912093, 179.0000000000000000 -12.9585501566365675, 180.0000000000000000 -14.0864721888845352, 181.0000000000000000 -14.8404926080210302, 182.0000000000000284 -15.5107986657318495, 183.0000000000000284 -16.0787374972979507, 184.0000000000000000 -16.5256562380002912, 185.0000000000000000 -16.8329020231198392, 186.0000000000000000 -16.9818219879375611, 187.0000000000000000 -16.9537632677344128, 188.0000000000000000 -16.7300729977913605, 189.0000000000000000 -16.2920983133893671, 190.0000000000000000 -15.6211863498093919, 191.0000000000000000 -14.5740105849692050, 192.0000000000000284 -13.0902888848280554, 193.0000000000000284 -11.2759349667291140, 193.9999999999999716 -9.2368625480155480, 195.0000000000000000 -7.0789853460305476, 196.0000000000000284 -4.9082170781172731, 197.0000000000000000 -2.8304714616189117, 197.9999999999999716 -0.9516622138786224, 199.0000000000000000 0.6222969477604061, 200.0000000000000000 1.7854923059550067, 201.0000000000000000 2.4829229495536316, 202.0000000000000284 2.7938692644205654, 203.0000000000000284 2.8138394787363814, 204.0000000000000000 2.6383418206816498, 205.0000000000000000 2.3628845184369451, 206.0000000000000284 2.0829758001828376, 207.0000000000000000 1.8941238940998992, 208.0000000000000000 1.8918370283687034, 209.0000000000000000 2.1716234311698228, 210.0000000000000000 2.8289913306838272, 211.0000000000000000 3.9885325921125663, 212.0000000000000000 5.6204193483809171, 213.0000000000000284 7.5885302292539958, 214.0000000000000000 9.7567438644969222, 215.0000000000000000 11.9889388838748072, 216.0000000000000284 14.1489939171527759, 217.0000000000000000 16.1007875940959373, 218.0000000000000000 17.7081985444694148, 219.0000000000000000 18.8351053980383227, 220.0000000000000000 19.3453867845677756, 221.0000000000000000 19.1592255837998486, 222.0000000000000284 18.3734418536126185, 223.0000000000000284 17.1168699909751822, 223.9999999999999716 15.5183443928566511, 225.0000000000000000 13.7066994562261328, 226.0000000000000000 11.8107695780527315, 227.0000000000000000 9.9593891553055656, 228.0000000000000000 8.2813925849537267, 229.0000000000000000 6.9056142639663349, 230.0000000000000000 5.9608885893124928, 231.0000000000000000 5.3338402769696627, 232.0000000000000284 4.8278770167351412, 233.0000000000000000 4.4470076663077780, 234.0000000000000000 4.1952410833864304, 235.0000000000000000 4.0765861256699507, 236.0000000000000284 4.0950516508571937, 236.9999999999999716 4.2546465166470133, 237.9999999999999716 4.5593795807382618, 239.0000000000000000 5.0132597008297948, 240.0000000000000000 5.6202957346204645, 241.0000000000000000 6.5151720719554955, 242.0000000000000000 7.7685560399500897, 243.0000000000000000 9.2884309022080664, 244.0000000000000000 10.9827799223332470, 245.0000000000000000 12.7595863639294542, 246.0000000000000000 14.5268334906005094, 246.9999999999999716 16.1925045659502302, 248.0000000000000000 17.6645828535824450, 249.0000000000000000 18.8510516171009712, 250.0000000000000000 19.6598941201096302, 251.0000000000000000 20.0726741532414401, 252.0000000000000284 20.1745741416293072, 253.0000000000000000 20.0290053006268671, 253.9999999999999716 19.6993788455877628, 255.0000000000000000 19.2491059918656333, 256.0000000000000000 18.7415979548141216, 256.9999999999999432 18.2402659497868775, 258.0000000000000000 17.8085211921375262, 259.0000000000000000 17.5097748972197209, 260.0000000000000000 17.4074382803871011, 261.0000000000000000 17.4821377689335655, 262.0000000000000000 17.6573504880260188, 263.0000000000000568 17.9107636998276192, 264.0000000000000000 18.2200646665015320, 265.0000000000000000 18.5629406502108907, 266.0000000000000000 18.9170789131188712, 266.9999999999999432 19.2601667173886106, 268.0000000000000568 19.5698913251832778, 269.0000000000000000 19.8239399986660203, 270.0000000000000000 20.0000000000000000, 271.0000000000000000 20.1049987496443201, 272.0000000000000000 20.1659239500552303, 273.0000000000000568 20.1905532863915482, 274.0000000000000000 20.1866644438121270, 275.0000000000000000 20.1620351074758055, 276.0000000000000000 20.1244429625414192, 277.0000000000000000 20.0816656941678069, 278.0000000000000000 20.0414809875138076, 278.9999999999999432 20.0116665277382566, 280.0000000000000000 20.0000000000000000, 281.0000000000000000 19.9999999999999964, 282.0000000000000000 20.0000000000000036, 283.0000000000000568 20.0000000000000000, 284.0000000000000000 20.0000000000000000, 285.0000000000000000 20.0000000000000000, 286.0000000000000000 20.0000000000000000, 287.0000000000000000 20.0000000000000000, 288.0000000000000000 20.0000000000000000, 289.0000000000000000 20.0000000000000000, 290.0000000000000000 20.0000000000000000, 291.0000000000000000 19.9999999999999964, 292.0000000000000000 20.0000000000000036, 293.0000000000000568 20.0000000000000000, 294.0000000000000000 20.0000000000000000, 295.0000000000000000 20.0000000000000000, 296.0000000000000000 20.0000000000000000, 297.0000000000000000 20.0000000000000000, 298.0000000000000000 20.0000000000000000, 298.9999999999999432 20.0000000000000000, 300.0000000000000000 20.0000000000000000, 301.0449999999999591 19.9999999999999964, 302.1599999999999682 20.0000000000000036, 303.3150000000000546 20.0000000000000000, 304.4799999999999613 20.0000000000000000, 305.6250000000000000 20.0000000000000000, 306.7199999999999704 20.0000000000000000, 307.7349999999999568 20.0000000000000000, 308.6400000000000432 20.0000000000000000, 309.4050000000000296 20.0000000000000000, 310.0000000000000000 20.0000000000000000)").expect("Failed to parse WKT");
        let back = LineString::try_from_wkt_str("LINESTRING (10.0000000000000000 10.0000000000000000, 10.5949999999999989 9.9999999999999982, 11.3600000000000012 10.0000000000000018, 12.2649999999999988 10.0000000000000000, 13.2799999999999976 10.0000000000000000, 14.3750000000000000 10.0000000000000000, 15.5200000000000014 10.0000000000000000, 16.6850000000000023 10.0000000000000000, 17.8399999999999999 10.0000000000000000, 18.9550000000000018 10.0000000000000000, 20.0000000000000000 10.0000000000000000, 20.9999999999999964 9.9999999999999982, 22.0000000000000036 10.0000000000000018, 23.0000000000000000 10.0000000000000000, 24.0000000000000000 10.0000000000000000, 25.0000000000000000 10.0000000000000000, 26.0000000000000036 10.0000000000000000, 26.9999999999999964 10.0000000000000000, 28.0000000000000000 10.0000000000000000, 29.0000000000000000 10.0000000000000000, 30.0000000000000000 10.0000000000000000, 31.0000000000000000 9.9999999999999982, 32.0000000000000000 10.0000000000000018, 32.9999999999999929 10.0000000000000000, 34.0000000000000000 10.0000000000000000, 35.0000000000000000 10.0000000000000000, 36.0000000000000071 10.0000000000000000, 37.0000000000000000 10.0000000000000000, 38.0000000000000000 10.0000000000000000, 39.0000000000000000 10.0000000000000000, 40.0000000000000000 10.0000000000000000, 40.9999999999999929 10.0101115064730823, 42.0000000000000071 10.0359520230154136, 42.9999999999999929 10.0707805453115924, 43.9999999999999929 10.1078560690462353, 45.0000000000000000 10.1404375899039518, 46.0000000000000071 10.1617841035693512, 47.0000000000000071 10.1651546057270465, 48.0000000000000000 10.1438080920616471, 49.0000000000000000 10.0910035582577606, 50.0000000000000000 10.0000000000000000, 51.0000000000000000 9.8705338096669184, 52.0000000000000071 9.7094492097218215, 53.0000000000000000 9.5206669409764473, 54.0000000000000000 9.3081077442425251, 55.0000000000000000 9.0756923603317947, 56.0000000000000071 8.8273415300559908, 57.0000000000000000 8.5669759942268566, 58.0000000000000000 8.2985164936561144, 59.0000000000000000 8.0258837691555112, 60.0000000000000000 7.7529985615367760, 61.0000000000000000 7.4963833923285241, 62.0000000000000000 7.2611514867233682, 62.9999999999999929 7.0341096410290467, 64.0000000000000142 6.8020646515532972, 65.0000000000000000 6.5518233146038636, 65.9999999999999858 6.2701924264884816, 67.0000000000000000 5.9439787835148925, 67.9999999999999858 5.5599891819908320, 69.0000000000000000 5.1050304182240431, 70.0000000000000000 4.5659092885222652, 70.9999999999999858 3.9163365104498835, 72.0000000000000142 3.1553887679642743, 73.0000000000000000 2.3079218069626561, 74.0000000000000000 1.3987913733422492, 75.0000000000000000 0.4528532130002744, 76.0000000000000000 -0.5050369281660509, 77.0000000000000000 -1.4500233042595019, 77.9999999999999858 -2.3572501693828669, 79.0000000000000000 -3.2018617776389191, 80.0000000000000000 -3.9590023831304411, 80.9999999999999858 -4.7425255197493295, 82.0000000000000142 -5.6414594798474571, 83.0000000000000000 -6.5937106552175724, 84.0000000000000000 -7.5371854376524210, 85.0000000000000000 -8.4097902189447495, 86.0000000000000000 -9.1494313908873064, 87.0000000000000000 -9.6940153452728364, 88.0000000000000000 -9.9814484738940905, 89.0000000000000000 -9.9496371685438110, 90.0000000000000000 -9.5364878210147488, 91.0000000000000000 -8.4870774578556585, 92.0000000000000142 -6.7241755764011524, 93.0000000000000000 -4.4522272746227527, 94.0000000000000142 -1.8756776504919834, 95.0000000000000000 0.8010281980196283, 96.0000000000000000 3.3734451729405639, 97.0000000000000000 5.6371281762992895, 97.9999999999999858 7.3876321101242954, 99.0000000000000000 8.4205118764440421, 100.0000000000000000 8.5313223772870170, 101.0000000000000000 7.4974024328047246, 102.0000000000000142 5.4070685261629237, 103.0000000000000142 2.5223420539114505, 104.0000000000000142 -0.8947555873998592, 105.0000000000000000 -4.5822030012211572, 106.0000000000000000 -8.2779787910026155, 107.0000000000000000 -11.7200615601943809, 108.0000000000000000 -14.6464299122466350, 109.0000000000000000 -16.7950624506095139, 110.0000000000000000 -17.9039377787331979, 111.0000000000000000 -17.8456588947919563, 112.0000000000000142 -16.8211755793787354, 113.0000000000000142 -15.0609866095717084, 114.0000000000000000 -12.7955907624490486, 115.0000000000000000 -10.2554868150889256, 116.0000000000000000 -7.6711735445695091, 116.9999999999999858 -5.2731497279689785, 117.9999999999999858 -3.2919141423654907, 119.0000000000000000 -1.9579655648372321, 120.0000000000000000 -1.5018027724623639, 121.0000000000000000 -1.9800207995830146, 122.0000000000000000 -3.1850255631230024, 123.0000000000000000 -4.9510321640330348, 124.0000000000000142 -7.1122557032638278, 125.0000000000000000 -9.5029112817660852, 126.0000000000000000 -11.9572140004905236, 127.0000000000000000 -14.3093789603878445, 128.0000000000000000 -16.3936212624087716, 129.0000000000000000 -18.0441560075040037, 130.0000000000000000 -19.0951982966242539, 131.0000000000000000 -19.6508790183537165, 132.0000000000000284 -19.9409737468458310, 133.0000000000000000 -19.9881646114051890, 134.0000000000000000 -19.8151337413363748, 135.0000000000000000 -19.4445632659439873, 136.0000000000000000 -18.8991353145326109, 137.0000000000000000 -18.2015320164068335, 138.0000000000000000 -17.3744355008712468, 139.0000000000000000 -16.4405278972304458, 140.0000000000000000 -15.4224913347890080, 140.9999999999999716 -14.0887695369765513, 142.0000000000000000 -12.2871979197529111, 143.0000000000000284 -10.1595461512180201, 143.9999999999999716 -7.8475838994718119, 145.0000000000000000 -5.4930808326142238, 146.0000000000000000 -3.2378066187451799, 147.0000000000000000 -1.2235309259646241, 147.9999999999999716 0.4079765776275239, 149.0000000000000000 1.5149462239313183, 150.0000000000000000 1.9556083448468335, 150.9999999999999716 1.6813479398860909, 152.0000000000000284 0.8121770719927385, 153.0000000000000284 -0.5407333317857214, 154.0000000000000000 -2.2662123444017874, 155.0000000000000000 -4.2530890388079534, 156.0000000000000284 -6.3901924879567247, 157.0000000000000000 -8.5663517648005829, 157.9999999999999716 -10.6703959422920498, 159.0000000000000000 -12.5911540933835990, 160.0000000000000000 -14.2174552910277434, 160.9999999999999716 -15.6846219948001693, 162.0000000000000284 -17.1831217176995992, 163.0000000000000000 -18.6845011128130736, 163.9999999999999716 -20.1603068332276294, 165.0000000000000000 -21.5820855320303089, 166.0000000000000284 -22.9213838623081507, 166.9999999999999716 -24.1497484771481936, 168.0000000000000000 -25.2387260296374798, 169.0000000000000284 -26.1598631728630409, 170.0000000000000000 -26.8847065599119261, 171.0000000000000000 -27.4030670396882741, 172.0000000000000000 -27.7387911472707991, 173.0000000000000000 -27.9144790650083756, 173.9999999999999716 -27.9527309752498851, 175.0000000000000000 -27.8761470603442092, 176.0000000000000284 -27.7073275026402293, 176.9999999999999716 -27.4688724844868304, 178.0000000000000000 -27.1833821882328870, 179.0000000000000000 -26.8734567962272806, 180.0000000000000000 -26.5616964908188962, 181.0000000000000000 -26.2807868131095752, 182.0000000000000284 -26.0254491709052154, 183.0000000000000284 -25.7613375466107541, 184.0000000000000000 -25.4541059226311361, 185.0000000000000000 -25.0694082813713237, 186.0000000000000000 -24.5728986052362437, 187.0000000000000000 -23.9302308766308585, 188.0000000000000000 -23.1070590779601055, 189.0000000000000000 -22.0690371916289365, 190.0000000000000000 -20.7818192000422961, 191.0000000000000000 -19.0782636836249466, 192.0000000000000284 -16.9011323904892201, 193.0000000000000284 -14.3809340545714139, 193.9999999999999716 -11.6481774098078255, 195.0000000000000000 -8.8333711901347591, 196.0000000000000284 -6.0670241294885079, 197.0000000000000000 -3.4796449618053824, 197.9999999999999716 -1.2017424210216623, 199.0000000000000000 0.6361747589263359, 200.0000000000000000 1.9035978441023218, 201.0000000000000000 2.5917062502638220, 202.0000000000000284 2.8309420435804400, 203.0000000000000284 2.6996904657339895, 204.0000000000000000 2.2763367584062850, 205.0000000000000000 1.6392661632791401, 206.0000000000000284 0.8668639220343674, 207.0000000000000000 0.0375152763537832, 208.0000000000000000 -0.7703945320808022, 209.0000000000000000 -1.4784802615875698, 210.0000000000000000 -2.0083566704847104, 211.0000000000000000 -2.4488776713700227, 212.0000000000000000 -2.9306949478012037, 213.0000000000000284 -3.4381199145362809, 214.0000000000000000 -3.9554639863332861, 215.0000000000000000 -4.4670385779502482, 216.0000000000000284 -4.9571551041451967, 217.0000000000000000 -5.4101249796761604, 218.0000000000000000 -5.8102596193011697, 219.0000000000000000 -6.1418704377782554, 220.0000000000000000 -6.3892688498654451, 221.0000000000000000 -6.5793899737663510, 222.0000000000000284 -6.7434674079931733, 223.0000000000000284 -6.8722602877668164, 223.9999999999999716 -6.9565277483081935, 225.0000000000000000 -6.9870289248382118, 226.0000000000000000 -6.9545229525777827, 227.0000000000000000 -6.8497689667478117, 228.0000000000000000 -6.6635261025692110, 229.0000000000000000 -6.3865534952628904, 230.0000000000000000 -6.0096102800497562, 231.0000000000000000 -5.4694875682872190, 232.0000000000000284 -4.7406524297595620, 233.0000000000000000 -3.8703779373280072, 234.0000000000000000 -2.9059371638537779, 235.0000000000000000 -1.8946031821980978, 236.0000000000000284 -0.8836490652221864, 236.9999999999999716 0.0796521142127278, 237.9999999999999716 0.9480272832454304, 239.0000000000000000 1.6742033690146922, 240.0000000000000000 2.2109072986592935, 241.0000000000000000 2.5369310705510819, 242.0000000000000000 2.6895163889749441, 243.0000000000000000 2.7090647399392171, 244.0000000000000000 2.6359776094522358, 245.0000000000000000 2.5106564835223373, 246.0000000000000000 2.3735028481578575, 246.9999999999999716 2.2649181893671315, 248.0000000000000000 2.2253039931584970, 249.0000000000000000 2.2950617455402900, 250.0000000000000000 2.5145929325208449, 251.0000000000000000 2.8942079437763857, 252.0000000000000284 3.3977369362295011, 253.0000000000000000 3.9958610467580788, 253.9999999999999716 4.6592614122400073, 255.0000000000000000 5.3586191695531715, 256.0000000000000000 6.0646154555754608, 256.9999999999999432 6.7479314071847600, 258.0000000000000000 7.3792481612589595, 259.0000000000000000 7.9292468546759460, 260.0000000000000000 8.3686086243136035, 261.0000000000000000 8.7101053078751374, 262.0000000000000000 8.9912371176926733, 263.0000000000000568 9.2197777525874631, 264.0000000000000000 9.4035009113807675, 265.0000000000000000 9.5501802928938471, 266.0000000000000000 9.6675895959479696, 266.9999999999999432 9.7635025193643816, 268.0000000000000568 9.8456927619643508, 269.0000000000000000 9.9219340225691361, 270.0000000000000000 10.0000000000000000, 271.0000000000000000 10.0660713507152977, 272.0000000000000000 10.1044090480439319, 273.0000000000000568 10.1199072661129499, 274.0000000000000000 10.1174601790494201, 275.0000000000000000 10.1019619609804003, 276.0000000000000000 10.0783067860329467, 277.0000000000000000 10.0513888283341206, 278.0000000000000000 10.0261022620109816, 278.9999999999999432 10.0073412611905894, 280.0000000000000000 10.0000000000000000, 281.0000000000000000 9.9999999999999982, 282.0000000000000000 10.0000000000000018, 283.0000000000000568 10.0000000000000000, 284.0000000000000000 10.0000000000000000, 285.0000000000000000 10.0000000000000000, 286.0000000000000000 10.0000000000000000, 287.0000000000000000 10.0000000000000000, 288.0000000000000000 10.0000000000000000, 289.0000000000000000 10.0000000000000000, 290.0000000000000000 10.0000000000000000, 291.0000000000000000 9.9999999999999982, 292.0000000000000000 10.0000000000000018, 293.0000000000000568 10.0000000000000000, 294.0000000000000000 10.0000000000000000, 295.0000000000000000 10.0000000000000000, 296.0000000000000000 10.0000000000000000, 297.0000000000000000 10.0000000000000000, 298.0000000000000000 10.0000000000000000, 298.9999999999999432 10.0000000000000000, 300.0000000000000000 10.0000000000000000, 301.0449999999999591 9.9999999999999982, 302.1599999999999682 10.0000000000000018, 303.3150000000000546 10.0000000000000000, 304.4799999999999613 10.0000000000000000, 305.6250000000000000 10.0000000000000000, 306.7199999999999704 10.0000000000000000, 307.7349999999999568 10.0000000000000000, 308.6400000000000432 10.0000000000000000, 309.4050000000000296 10.0000000000000000, 310.0000000000000000 10.0000000000000000, 10.0000000000000000 10.0000000000000000)").expect("Failed to parse WKT");;
        let clipped = front.clipwith(&back);
        println!("Clipped: {:?}", clipped);
    }

    #[test]
    fn test_clip_simple() {
        let joydivfront = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 100.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 0.0},
            coord! {x: 60.0, y: 0.0},
        ]);
        let joydivback = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 10.0},
            coord! {x: 10.0, y: 10.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 40.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 10.0},
            coord! {x: 60.0, y: 10.0},
        ]);
        let clipped = joydivback.clipwith(&joydivfront);
        println!("Clipped: {:?}", clipped);
    }

/*    #[test]
    fn test_experiment1_geos_clip_line() {
        let joydivfront = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 100.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 0.0},
            coord! {x: 60.0, y: 0.0},
        ]);
        let joydivback = LineString::<f64>::new(vec![
            coord! {x: 0.0, y: 10.0},
            coord! {x: 10.0, y: 10.0},
            coord! {x: 20.0, y: 10.0},
            coord! {x: 30.0, y: 40.0},
            coord! {x: 40.0, y: 10.0},
            coord! {x: 50.0, y: 10.0},
            coord! {x: 60.0, y: 10.0},
        ]);


        let geo_perim_front: geos::Geometry = Polygon::new(joydivfront.clone(), vec![])
            .try_into()
            .expect("Invalid geometry");
        // let hatch_lines: Vec<geo_types::LineString<f64>> = hatch_lines.iter().map(|x| x.clone()).collect();
        let geo_clipped_lines: Vec<Geometry> = vec![joydivback.clone().try_into().expect("Couldn't convert back line")];
        println!("geo_clipped_lines is {:?}", &joydivback);
        // (&hatch_lines).iter()
        // .map(|hatch_line|
        //     (hatch_line).try_into().expect("Invalid hatch lines")).collect();
        let geo_clipped_lines_collection = Geometry::create_geometry_collection(geo_clipped_lines).expect("Got this far?");
        // let _clipped_object = geo_perim_front.difference(&geo_clipped_lines_collection).expect("Got this far?");
        let _clipped_object = geo_clipped_lines_collection.difference(&geo_perim_front).expect("Got this far?");
        println!("CLipped object is: {}", _clipped_object.to_wkt().expect("As a string!"))
    }
*/}
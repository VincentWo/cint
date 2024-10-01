use cint::{replicate, Dynamic};

#[test]
fn decode() {
    fn decode_bitmask(
        imm_n: Dynamic,
        imms: Dynamic,
        immr: Dynamic,
        immediate: bool,
        m: u8,
    ) -> (u64, u64) {
        assert_eq!(imm_n.bits(), 1);
        assert_eq!(imms.bits(), 6);
        assert_eq!(immr.bits(), 6);

        let len = (imm_n.concat(!imms)).highest_set_bit();

        assert!(len >= 1);
        assert!(m >= (1 << len));

        let levels = Dynamic::ones(len).zero_extend(6);

        if immediate && (imms & levels) == levels {
            panic!()
        }

        let s = imms & levels;
        let r = immr & levels;
        let diff = s - r;

        let esize = 1 << len;

        let d = diff & Dynamic::ones(len);

        let welem = Dynamic::ones(u8::from(s) + 1).zero_extend(esize);
        let telem = Dynamic::ones(u8::from(r) + 1).zero_extend(esize);

        let wmask = replicate(welem.rotate_right(r.into()), m / esize);
        let tmask = replicate(telem, m / esize);

        (wmask, tmask)
    }

    let (wmask, _) = decode_bitmask(
        Dynamic::new(0, 1),
        Dynamic::new(0b110000, 6),
        Dynamic::new(0b000001, 6),
        true,
        64,
    );

    assert_eq!(wmask, 0x8080808080808080);
}

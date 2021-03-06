extern crate prusti_contracts;

fn test() {
    let mut i = 0;

    #[invariant="i < 10"]
    while i < 10 {
        #[invariant="i < 10"]
        'inner: while {
            i += 1;
            i -= 1;
            i < 10
        } {
            i += 1;
            if i == 234 {
                break;
            }
            if i == 345 {
                continue;
            }
        }
        assert!(i == 10);
    }

    assert!(i == 10);
}

fn main() {}

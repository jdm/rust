// pp-exact - Make sure we actually print the attributes

class cat {
    #[cat_maker]
    new(name: ~str) { self.name = name; }
    #[cat_dropper]
    drop { #error["%s landed on hir feet", self.name]; }
    let name: ~str;
}

fn main() { let _kitty = cat(~"Spotty"); }

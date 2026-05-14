struct JenkinsRng {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl JenkinsRng {
    pub const fn new(seed: u64) -> Self {
        let mut rng = JenkinsRng {
            a: 0xf1ea_5eed,
            b: seed,
            c: seed,
            d: seed,
        };
        // 20 Runden "warmdrehen"#
        let mut i = 0;
        while i < 20 {
            rng.next();
            i += 1;
        }
        rng
    }

    const fn next(&mut self) -> u64 {
        let e = self.a.wrapping_sub(self.b.rotate_left(7));
        self.a = self.b ^ self.c.rotate_left(13);
        self.b = self.c.wrapping_add(self.d.rotate_left(37));
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }
}
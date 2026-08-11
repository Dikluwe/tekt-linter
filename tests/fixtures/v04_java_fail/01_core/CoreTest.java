// @prompt 00_nucleo/prompts/core.md
// @layer L1
// @updated 2026-06-08
package core;

import java.io.File;

public class CoreTest {
    public void read() {
        File f = new File("test.txt");
    }

    @org.junit.Test
    public void testRead() {}
}

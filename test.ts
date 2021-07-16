//@deno-types="./pkg/tar_wasm.d.ts"
import initWasm, { StreamingTarPacker } from "./pkg/tar_wasm.js";

(async () => {
  await initWasm(Deno.readFile("./pkg/tar_wasm_bg.wasm"));

  const file = await Deno.open("./test.tar", {
    create: true,
    write: true,
    truncate: true,
  });
  console.log("opened './test.tar'");

  let writtenBytes = 0;
  let totalBytes = 0;
  const packer = new StreamingTarPacker(
    new WritableStream({
      write: async (chunk: Uint8Array) => {
        writtenBytes += chunk.length;
        console.log(
          `streamed ${chunk.length} bytes (${(
            (writtenBytes / totalBytes) *
            100
          ).toFixed(0)}%)`
        );
        await file.write(chunk);
      },
      close: () => {
        console.log("closed stream");
        file.close();
      },
    })
  );

  const res = await fetch("https://www.everypixel.com/i/free_1.jpg");
  const size = parseInt(await res.headers.get("content-length")!)!;
  console.log("downloading len", size);
  totalBytes = size;
  await packer.add_file_stream("./cat.png", {}, size, res.body!);
  console.log("done!");
})();

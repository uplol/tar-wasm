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
          ).toFixed(0)}%) (${writtenBytes})`
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

  const res2 = await fetch(
    "https://media.discordapp.net/attachments/638480814887141386/865368782834499624/unknown.png"
  );
  const size2 = parseInt(await res2.headers.get("content-length")!)!;

  totalBytes = size;

  console.log("downloading len", size);
  await packer.addFileStream("./cat.jpg", {}, size, res.body!);
  console.log("done with first!");

  writtenBytes = 0;
  totalBytes = size2;

  {
    console.log("downloading len", size2);
    await packer.addFileStream("./dir/buildyboi.png", {}, size2, res2.body!);
    console.log("done with second!");
  }

  await packer.finish();
})();

import { assert, assertEquals } from "jsr:@std/assert";

Deno.test("auth custom x-rdfs-token works", async () => {
  let _ = await fetch("http://localhost:8888/", {
    headers: {
      "x-rdfs-token": "695bfaf2-f381-470b-945c-6cb11fa7a73c",
    },
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => assertEquals(data.status, 200));
});

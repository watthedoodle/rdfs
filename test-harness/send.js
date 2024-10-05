import { assert, assertEquals } from "jsr:@std/assert";

/* -------------------------------------------------------------------------------------------------
WARNING: we are assuming that there are two worker nodes is already running on two custom ports, #
later on we may need some automations in order to spin up the worker node before running this test
------------------------------------------------------------------------------------------------- */
const Token = "695bfaf2-f381-470b-945c-6cb11fa7a73c";

Deno.test("can call send-chunk", async () => {
  let _ = await fetch("http://localhost:8888/send-chunk", {
    method: "POST",
    headers: {
      "x-rdfs-token": Token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ "id": "README.md", "target": "http://localhost:9999" }),
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => {
      console.log(data.body);
      assertEquals(data.status, 200);
    });
});

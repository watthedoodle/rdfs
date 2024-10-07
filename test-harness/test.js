import { assert, assertEquals } from "jsr:@std/assert";

/* -------------------------------------------------------------------------------------------------
WARNING: we are assuming that the worker node is already running, later on we may need
some automations in order to spin up the worker node before running this test
------------------------------------------------------------------------------------------------- */
const Token = "695bfaf2-f381-470b-945c-6cb11fa7a73c";

Deno.test("x-rdfs-token", async () => {
  let _ = await fetch("http://localhost:8888/", {
    headers: {
      "x-rdfs-token": Token,
    },
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => assertEquals(data.status, 200));
});

Deno.test("get-chunk-404", async () => {
  let _ = await fetch("http://localhost:8888/get-chunk", {
    method: "POST",
    headers: {
      "x-rdfs-token": Token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ "id": "this-file-does-not-exist" }),
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => {
      // console.log(data.body)
      assertEquals(data.status, 404);
    });
});

Deno.test("get-chunk-ok", async () => {
  let _ = await fetch("http://localhost:8888/get-chunk", {
    method: "POST",
    headers: {
      "x-rdfs-token": Token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ "id": "README.md" }),
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => {
      console.log(data.body);
      assertEquals(data.status, 200);
    });
});

Deno.test("store-chunk", async () => {
  let _ = await fetch("http://localhost:8888/store-chunk", {
    method: "POST",
    headers: {
      "x-rdfs-token": Token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      "id": "test.txt",
      "chunk": "dGhpcyBpcyBhIHRlc3QgZmlsZSE=",
    }),
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => {
      console.log(data.body);
      assertEquals(data.status, 200);
    });
});

Deno.test("delete-chunk", async () => {
  let _ = await fetch("http://localhost:8888/delete-chunk", {
    method: "POST",
    headers: {
      "x-rdfs-token": Token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ "id": "test.txt" }),
  }).then((x) => x.text().then((data) => ({ status: x.status, body: data })))
    .then((data) => {
      // console.log(data.body);
      assertEquals(data.status, 200);
    });
});

Deno.test("send-chunk", async () => {
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

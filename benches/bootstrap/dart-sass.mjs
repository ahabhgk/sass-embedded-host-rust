import sass from "sass";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

sass.compile(path.resolve(__dirname, "./bootstrap/scss/bootstrap.scss"), {
	style: "expanded",
	sourceMap: true,
});
sass.compile(path.resolve(__dirname, "./bootstrap/scss/bootstrap-grid.scss"), {
	style: "expanded",
	sourceMap: true,
});
sass.compile(path.resolve(
	__dirname,
	"./bootstrap/scss/bootstrap-reboot.scss",
), {
	style: "expanded",
	sourceMap: true,
});
sass.compile(path.resolve(
	__dirname,
	"./bootstrap/scss/bootstrap-utilities.scss",
), {
	style: "expanded",
	sourceMap: true,
});

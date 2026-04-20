function setTheme() {
	let radioSelectors = document.getElementsByName("theme");
	for (const radio in radioSelectors) {
		if (radio.checked) {
			return;
		}
	}
	let theme;
	for (const cookie of document.cookie.split(";")) {
		const [name, value] = cookie.trim().split("=");
		if (name === "theme") {
			theme = value;
		}
	}
	if (!theme) {
		const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
		if (darkModeQuery.matches) {
			theme = "dark";
		} else {
			theme = "light";
		}
	}
	if (theme === "light") {
		document.getElementById("light").checked = true;
		document.body.classList.add("light");
	} else if (theme === "dark") {
		document.getElementById("dark").checked = true;
		document.body.classList.add("dark");
	}
	updateTheme(theme);
}

function updateTheme(theme) {
	if (theme !== "light" && theme !== "dark") {
		return;
	}
	if (theme === "light") {
		document.body.classList.remove("dark");
		document.body.classList.add("light");
	} else if (theme === "dark") {
		document.body.classList.remove("light");
		document.body.classList.add("dark");
	}
	document.cookie = `theme=${theme};path=/;Max-Age=2592000;Secure=true;SameSite=Lax;`;
}

window.onload = setTheme;

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
			document.getElementById("dark").checked = true;
			theme = "dark";
		} else {
			document.getElementById("light").checked = true;
			theme = "light";
		}
	}
	updateTheme(theme);
}

function updateTheme(theme) {
	if (theme !== "light" && theme !== "dark") {
		return;
	}
	document.cookie = `theme=${theme};path=/;Max-Age=2592000;Secure=true;SameSite=Lax;`;
}

window.onload = setTheme;

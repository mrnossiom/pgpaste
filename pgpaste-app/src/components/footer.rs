use leptos::{component, view, IntoView, Scope};

#[component]
pub(crate) fn Footer(cx: Scope) -> impl IntoView {
	view! { cx,
		<footer class="footer">
			<div class="column content has-text-centered">
				<p>
					<strong>"PGPaste"</strong>" by "<a href="https://github.com/MrNossiom">"Milo Moisson"</a>"." <br />
					"The source code is licensed "<a href="http://opensource.org/licenses/mit-license.php">"MIT"</a>"." <br />
					"The website content is licensed "<a href="http://creativecommons.org/licenses/by-nc-sa/4.0/">"CC BY NC SA 4.0"</a>"."
				</p>
			</div>
		</footer>
	}
}

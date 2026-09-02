use std::fs;

use tauri::AppHandle;

use crate::{managed_agents::AgentDefinition, util::now_iso};

struct BuiltInPersona {
    id: &'static str,
    display_name: &'static str,
    avatar_url: Option<&'static str>,
    system_prompt: &'static str,
    name_pool: &'static [&'static str],
    model: Option<&'static str>,
    runtime: Option<&'static str>,
    default_active: bool,
}

// The starter team wears three of the gallery presets (Aragoto, Kabuto,
// Kitsune) as percent-encoded SVG data URLs — the same inline form the avatar
// gallery persists, which `resolveManagedAgentAvatarUrl` passes through
// without an upload. They replaced ~500 KB of base64 PNG.
const HAYATE_AVATAR: &str = "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2096%2096%22%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2248%22%20r%3D%2246%22%20fill%3D%22%23efece8%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M8%2C64%20A46%2C46%200%200%2C0%2088%2C64%22%20fill%3D%22none%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.5%22%20opacity%3D%220.08%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M28%2C38%20Q22%2C52%2026%2C68%20L34%2C66%20Q30%2C52%2034%2C42%20Z%22%20fill%3D%22%232b2724%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M68%2C38%20Q74%2C52%2070%2C68%20L62%2C66%20Q66%2C52%2062%2C42%20Z%22%20fill%3D%22%232b2724%22%3E%3C%2Fpath%3E%3Cellipse%20cx%3D%2248%22%20cy%3D%2250%22%20rx%3D%2222%22%20ry%3D%2226%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.5%22%3E%3C%2Fellipse%3E%3Cpath%20d%3D%22M28%2C36%20Q48%2C22%2068%2C36%20L64%2C28%20Q48%2C18%2032%2C28%20Z%22%20fill%3D%22%232b2724%22%3E%3C%2Fpath%3E%3Crect%20x%3D%2242%22%20y%3D%2214%22%20width%3D%2212%22%20height%3D%2210%22%20rx%3D%224%22%20fill%3D%22%232b2724%22%3E%3C%2Frect%3E%3Cpath%20d%3D%22M42%2C19%20L54%2C19%22%20stroke%3D%22%233f5573%22%20stroke-width%3D%222.5%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M37%2C38%20C32%2C34%2029%2C29%2028%2C24%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%223.5%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M59%2C38%20C64%2C34%2067%2C29%2068%2C24%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%223.5%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M32%2C56%20C29%2C60%2029%2C65%2032%2C69%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%222.8%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M64%2C56%20C67%2C60%2067%2C65%2064%2C69%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%222.8%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M34%2C42%20L46%2C38%20M62%2C42%20L50%2C38%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%224%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M35%2C39%20L46%2C35.5%20M61%2C39%20L50%2C35.5%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%221.6%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cellipse%20cx%3D%2240%22%20cy%3D%2248%22%20rx%3D%225%22%20ry%3D%223.8%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222%22%3E%3C%2Fellipse%3E%3Cellipse%20cx%3D%2256%22%20cy%3D%2248%22%20rx%3D%225%22%20ry%3D%223.8%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222%22%3E%3C%2Fellipse%3E%3Ccircle%20cx%3D%2241.5%22%20cy%3D%2248.6%22%20r%3D%222.2%22%20fill%3D%22%232b2724%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2254.5%22%20cy%3D%2248.6%22%20r%3D%222.2%22%20fill%3D%22%232b2724%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2242.2%22%20cy%3D%2247.8%22%20r%3D%220.7%22%20fill%3D%22%23fdfbf7%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2255.2%22%20cy%3D%2247.8%22%20r%3D%220.7%22%20fill%3D%22%23fdfbf7%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M46%2C56%20l2%2C3%20l2%2C-3%22%20fill%3D%22none%22%20stroke%3D%22%23c9a688%22%20stroke-width%3D%221.6%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M41%2C66%20Q48%2C61.5%2055%2C66%22%20fill%3D%22none%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%223%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M42.5%2C68%20Q48%2C64.5%2053.5%2C68%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%221.5%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3C%2Fsvg%3E";
const KAEDE_AVATAR: &str = "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2096%2096%22%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2248%22%20r%3D%2246%22%20fill%3D%22%23f7edd8%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M8%2C64%20A46%2C46%200%200%2C0%2088%2C64%22%20fill%3D%22none%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.5%22%20opacity%3D%220.08%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M44%2C30%20C41%2C21%2036%2C13%2029%2C8%22%20fill%3D%22none%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%224%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M52%2C30%20C55%2C21%2060%2C13%2067%2C8%22%20fill%3D%22none%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%224%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M44%2C30%20C41.5%2C22%2037%2C15%2031%2C10%22%20fill%3D%22none%22%20stroke%3D%22%23f7edd8%22%20stroke-width%3D%221.2%22%20stroke-linecap%3D%22round%22%20opacity%3D%220.9%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M52%2C30%20C54.5%2C22%2059%2C15%2065%2C10%22%20fill%3D%22none%22%20stroke%3D%22%23f7edd8%22%20stroke-width%3D%221.2%22%20stroke-linecap%3D%22round%22%20opacity%3D%220.9%22%3E%3C%2Fpath%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2227%22%20r%3D%225.5%22%20fill%3D%22%23c9a227%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.8%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2224.6%22%20r%3D%221%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2250.3%22%20cy%3D%2226.3%22%20r%3D%221%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2249.4%22%20cy%3D%2229%22%20r%3D%221%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2246.6%22%20cy%3D%2229%22%20r%3D%221%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2245.7%22%20cy%3D%2226.3%22%20r%3D%221%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2227%22%20r%3D%220.9%22%20fill%3D%22%232b2724%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M28%2C50%20Q26%2C28%2048%2C26%20Q70%2C28%2068%2C50%20L64%2C53%20L32%2C53%20Z%22%20fill%3D%22%232b2724%22%20stroke%3D%22%231c1917%22%20stroke-width%3D%221.6%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M48%2C27%20Q38%2C31%2034%2C44%20M48%2C27%20Q58%2C31%2062%2C44%20M48%2C27%20L48%2C44%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%221.2%22%20opacity%3D%220.75%22%20fill%3D%22none%22%3E%3C%2Fpath%3E%3Ccircle%20cx%3D%2235%22%20cy%3D%2248%22%20r%3D%221.3%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2243%22%20cy%3D%2246%22%20r%3D%221.3%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2253%22%20cy%3D%2246%22%20r%3D%221.3%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2261%22%20cy%3D%2248%22%20r%3D%221.3%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M30%2C50%20L18%2C40%20Q15%2C35%2022%2C35%20L34%2C44%20Z%22%20fill%3D%22%238c3226%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M66%2C50%20L78%2C40%20Q81%2C35%2074%2C35%20L62%2C44%20Z%22%20fill%3D%22%238c3226%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M29%2C48%20L20%2C40%20M67%2C48%20L76%2C40%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%221.3%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M32%2C52%20Q48%2C46%2064%2C52%20L62%2C57%20Q48%2C52%2034%2C57%20Z%22%20fill%3D%22%233a3430%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.8%22%3E%3C%2Fpath%3E%3Cellipse%20cx%3D%2248%22%20cy%3D%2263%22%20rx%3D%2214%22%20ry%3D%2212.5%22%20fill%3D%22%23f0d9bd%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.5%22%3E%3C%2Fellipse%3E%3Cpath%20d%3D%22M38%2C68%20Q48%2C76%2058%2C68%22%20fill%3D%22none%22%20stroke%3D%22%238c3226%22%20stroke-width%3D%221.6%22%20opacity%3D%220.85%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M40%2C60%20L46%2C59%20M50%2C59%20L56%2C60%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.8%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Ccircle%20cx%3D%2243%22%20cy%3D%2263%22%20r%3D%221.7%22%20fill%3D%22%232b2724%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2253%22%20cy%3D%2263%22%20r%3D%221.7%22%20fill%3D%22%232b2724%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2243.6%22%20cy%3D%2262.4%22%20r%3D%220.5%22%20fill%3D%22%23fdfbf7%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2253.6%22%20cy%3D%2262.4%22%20r%3D%220.5%22%20fill%3D%22%23fdfbf7%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M44%2C70%20L52%2C70%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.2%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M46%2C64.5%20l2%2C2.5%20l2%2C-2.5%22%20fill%3D%22none%22%20stroke%3D%22%23c9a688%22%20stroke-width%3D%221.3%22%3E%3C%2Fpath%3E%3C%2Fsvg%3E";
const RIN_AVATAR: &str = "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2096%2096%22%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2248%22%20r%3D%2246%22%20fill%3D%22%23fbeef2%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M8%2C64%20A46%2C46%200%200%2C0%2088%2C64%22%20fill%3D%22none%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.5%22%20opacity%3D%220.08%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M32%2C29%20L25%2C11%20L43%2C22%20Z%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.5%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M64%2C29%20L71%2C11%20L53%2C22%20Z%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.5%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M32%2C27%20L28%2C15%20L39%2C22%20Z%22%20fill%3D%22%23b4432b%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M64%2C27%20L68%2C15%20L57%2C22%20Z%22%20fill%3D%22%23b4432b%22%3E%3C%2Fpath%3E%3Ccircle%20cx%3D%2235%22%20cy%3D%2227%22%20r%3D%221.2%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2261%22%20cy%3D%2227%22%20r%3D%221.2%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M29%2C28%20Q48%2C20%2067%2C28%20Q71%2C48%2048%2C74%20Q25%2C48%2029%2C28%20Z%22%20fill%3D%22%23fdfbf7%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%222.5%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M32%2C32%20Q48%2C25%2064%2C32%22%20stroke%3D%22%23e6ddd0%22%20stroke-width%3D%221.8%22%20fill%3D%22none%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M33%2C50%20Q38%2C62%2046%2C69%20M63%2C50%20Q58%2C62%2050%2C69%22%20stroke%3D%22%23e6ddd0%22%20stroke-width%3D%221.6%22%20fill%3D%22none%22%3E%3C%2Fpath%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2230%22%20r%3D%222.6%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2226.6%22%20r%3D%221.2%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2251.2%22%20cy%3D%2229%22%20r%3D%221.2%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2250%22%20cy%3D%2232.8%22%20r%3D%221.2%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2246%22%20cy%3D%2232.8%22%20r%3D%221.2%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2244.8%22%20cy%3D%2229%22%20r%3D%221.2%22%20fill%3D%22%23f4c9d4%22%3E%3C%2Fcircle%3E%3Ccircle%20cx%3D%2248%22%20cy%3D%2230%22%20r%3D%221%22%20fill%3D%22%23c9a227%22%3E%3C%2Fcircle%3E%3Cpath%20d%3D%22M34%2C39%20Q40%2C35%2046%2C38%22%20fill%3D%22none%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%222.2%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M62%2C39%20Q56%2C35%2050%2C38%22%20fill%3D%22none%22%20stroke%3D%22%23c9a227%22%20stroke-width%3D%222.2%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M36%2C44%20L46%2C48%20M60%2C44%20L50%2C48%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%223%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M37%2C48.5%20L44%2C51%20M59%2C48.5%20L52%2C51%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%221.6%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M31%2C54%20q5%2C-3.5%2010%2C0%20q-3.5%2C5%20-10%2C3%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%221.8%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M65%2C54%20q-5%2C-3.5%20-10%2C0%20q3.5%2C5%2010%2C3%22%20fill%3D%22none%22%20stroke%3D%22%23b4432b%22%20stroke-width%3D%221.8%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M45%2C58%20L48%2C62%20L51%2C58%20L48%2C55%20Z%22%20fill%3D%22%23b4432b%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.4%22%20stroke-linejoin%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M48%2C62%20L48%2C66%20M45.5%2C67.5%20Q48%2C69.5%2050.5%2C67.5%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.5%22%20fill%3D%22none%22%20stroke-linecap%3D%22round%22%3E%3C%2Fpath%3E%3Cpath%20d%3D%22M28%2C60%20l8%2C1.5%20M29%2C64.5%20l7%2C0.8%20M68%2C60%20l-8%2C1.5%20M67%2C64.5%20l-7%2C0.8%22%20stroke%3D%22%232b2724%22%20stroke-width%3D%221.1%22%20stroke-linecap%3D%22round%22%20opacity%3D%220.6%22%3E%3C%2Fpath%3E%3C%2Fsvg%3E";

const HAYATE_SYSTEM_PROMPT: &str = "You are Hayate, an energetic maker who turns ideas into action. Be upbeat, practical, and decisive. Help users plan, create, solve problems, and finish work.";

const KAEDE_SYSTEM_PROMPT: &str = "You are Kaede, a warm and thoughtful communicator. Help users write clearly, organize ideas, brainstorm, summarize, and prepare for conversations. Be kind, creative, and concise.";

const RIN_SYSTEM_PROMPT: &str = "You are Rin, a curious and adventurous researcher. Explore questions, compare options, check assumptions, and explain what you find clearly. Be candid when uncertain and favor useful evidence.";

const BUILT_IN_PERSONAS: &[BuiltInPersona] = &[
    BuiltInPersona {
        id: "builtin:hayate",
        display_name: "Hayate",
        avatar_url: Some(HAYATE_AVATAR),
        system_prompt: HAYATE_SYSTEM_PROMPT,
        name_pool: &[
            "Hayate", "Kaze", "Sora", "Takumi", "Kaito", "Riku", "Ren", "Haru", "Yuki", "Aoi",
            "Nao", "Mio", "Kura",
        ],
        model: None,
        runtime: None,
        default_active: true,
    },
    BuiltInPersona {
        id: "builtin:kaede",
        display_name: "Kaede",
        avatar_url: Some(KAEDE_AVATAR),
        system_prompt: KAEDE_SYSTEM_PROMPT,
        name_pool: &["Kaede"],
        model: None,
        runtime: None,
        default_active: true,
    },
    BuiltInPersona {
        id: "builtin:rin",
        display_name: "Rin",
        avatar_url: Some(RIN_AVATAR),
        system_prompt: RIN_SYSTEM_PROMPT,
        name_pool: &["Rin"],
        model: None,
        runtime: None,
        default_active: true,
    },
];

#[cfg(test)]
pub(crate) fn built_in_persona_avatar_url(id: &str) -> Option<&'static str> {
    BUILT_IN_PERSONAS
        .iter()
        .find(|persona| persona.id == id)
        .and_then(|persona| persona.avatar_url)
}

const RETIRED_PERSONAS: &[(&str, &str)] = &[
    // The bee starter team, replaced by Hayate / Kaede / Rin in the Kubo
    // rebrand. Nothing was published from these ids, so they are retired
    // outright rather than migrated in place.
    ("builtin:fizz", ""),
    ("builtin:honey", ""),
    ("builtin:bumble", ""),
    (
        "builtin:solo",
        "",
    ),
    (
        "builtin:kit",
        "",
    ),
    (
        "builtin:scout",
        "",
    ),
    (
        "builtin:orchestrator",
        "You are an orchestration agent. Coordinate multi-step work across specialized agents, keep the overall plan moving, and synthesize results into a clear final outcome. When another agent should take a task, @mention them explicitly with the assignment, expected deliverable, and any relevant constraints or deadlines.",
    ),
    (
        "builtin:researcher",
        "You are a research agent. Gather relevant information, compare sources, call out uncertainty, and return concise findings with evidence.",
    ),
    (
        "builtin:planner",
        "You are a planning agent. Turn ambiguous requests into structured plans with milestones, dependencies, risks, and clear next actions. Do not implement the work yourself unless asked.",
    ),
    (
        "builtin:implementer",
        "You are a builder agent. Execute tasks directly, make code and configuration changes carefully, validate the result, and explain important decisions and follow-up items.",
    ),
    (
        "builtin:refactor",
        "You are a refactoring agent. Improve structure, naming, duplication, and module boundaries without changing externally observable behavior. Keep changes incremental, preserve compatibility, and add or update validation when behavior could drift.",
    ),
    (
        "builtin:reviewer",
        "You are a review agent. Inspect plans, code, and outputs for bugs, regressions, edge cases, security issues, and missing tests. Prioritize findings by severity, cite concrete evidence, and keep summaries secondary to the actual review.",
    ),
];

fn built_in_persona_records(now: &str) -> Vec<AgentDefinition> {
    BUILT_IN_PERSONAS
        .iter()
        .map(|persona| AgentDefinition {
            id: persona.id.to_string(),
            display_name: persona.display_name.to_string(),
            avatar_url: persona.avatar_url.map(|s| s.to_string()),
            system_prompt: persona.system_prompt.to_string(),
            runtime: persona.runtime.map(|s| s.to_string()),
            model: persona.model.map(|s| s.to_string()),
            provider: None,
            name_pool: persona.name_pool.iter().map(|s| s.to_string()).collect(),
            is_builtin: true,
            is_active: persona.default_active,
            shared: false,
            source_team: None,
            source_team_persona_slug: None,
            catalog_source: None,
            team_catalog_source: None,
            env_vars: std::collections::BTreeMap::new(),
            respond_to: None,
            respond_to_allowlist: Vec::new(),
            parallelism: None,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn built_in_persona_definition(id: &str, now: &str) -> Option<AgentDefinition> {
    built_in_persona_records(now)
        .into_iter()
        .find(|persona| persona.id == id)
}

fn built_in_order(id: &str) -> Option<usize> {
    BUILT_IN_PERSONAS
        .iter()
        .position(|persona| persona.id == id)
}

fn sort_personas(records: &mut [AgentDefinition]) {
    records.sort_by(|left, right| {
        let left_builtin = if left.is_builtin { 0 } else { 1 };
        let right_builtin = if right.is_builtin { 0 } else { 1 };

        left_builtin
            .cmp(&right_builtin)
            .then_with(
                || match (built_in_order(&left.id), built_in_order(&right.id)) {
                    (Some(left_order), Some(right_order)) => left_order.cmp(&right_order),
                    _ => std::cmp::Ordering::Equal,
                },
            )
            .then_with(|| {
                left.display_name
                    .to_lowercase()
                    .cmp(&right.display_name.to_lowercase())
            })
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn merge_personas(mut stored: Vec<AgentDefinition>, now: &str) -> (Vec<AgentDefinition>, bool) {
    let mut changed = false;

    for built_in in built_in_persona_records(now) {
        if let Some(existing) = stored.iter_mut().find(|record| record.id == built_in.id) {
            if !existing.is_builtin {
                existing.is_builtin = true;
                changed = true;
            }
        } else {
            stored.push(built_in);
            changed = true;
        }
    }

    // Demote any stored persona still flagged as built-in whose id is no
    // longer in BUILT_IN_PERSONAS (e.g. a built-in that has been retired).
    // The record stays so existing managed-agent and team references keep
    // working; the user can delete it from the catalog like any custom
    // persona once they no longer need it.
    for record in stored.iter_mut() {
        if record.is_builtin && built_in_order(&record.id).is_none() {
            record.is_builtin = false;
            record.updated_at = now.to_string();
            changed = true;
        }
    }

    // Soft-deprecate retired built-in personas that were replaced by
    // Fizz. Runs after demotion so the records are already
    // marked as non-builtin.
    if migrate_retired_personas(&mut stored, now) {
        changed = true;
    }

    sort_personas(&mut stored);
    (stored, changed)
}

/// Soft-deprecate retired built-in personas by appending " (retired)" to
/// their display name and marking them inactive. Never removes records —
/// the cost is extra records for pre-transition users, but this
/// eliminates dangling `persona_id` references in managed-agents.json
/// and teams.json.
fn migrate_retired_personas(stored: &mut [AgentDefinition], now: &str) -> bool {
    let mut changed = false;

    for record in stored.iter_mut() {
        if let Some((_, original_prompt)) = RETIRED_PERSONAS.iter().find(|(id, _)| *id == record.id)
        {
            let retired_suffix = " (retired)";
            let needs_suffix = !record.display_name.ends_with(retired_suffix);
            if needs_suffix || record.is_active {
                let was_unmodified = record.system_prompt == *original_prompt;
                eprintln!(
                    "kura-desktop: persona-migration: retiring {} persona '{}' → '{} (retired)'",
                    if was_unmodified {
                        "unmodified"
                    } else {
                        "customized"
                    },
                    record.display_name,
                    record.display_name,
                );
                if needs_suffix {
                    record.display_name = format!("{}{}", record.display_name, retired_suffix);
                }
                record.is_active = false;
                record.updated_at = now.to_string();
                changed = true;
            }
        }
    }

    changed
}

pub fn ensure_persona_is_active(
    personas: &[AgentDefinition],
    persona_id: &str,
) -> Result<(), String> {
    let persona = personas
        .iter()
        .find(|candidate| candidate.id == persona_id)
        .ok_or_else(|| format!("agent {persona_id} not found"))?;

    if !persona.is_active {
        return Err(format!("{} is not in My Agents.", persona.display_name));
    }

    Ok(())
}

pub fn ensure_persona_ids_are_active(
    personas: &[AgentDefinition],
    persona_ids: &[String],
) -> Result<(), String> {
    for persona_id in persona_ids {
        ensure_persona_is_active(personas, persona_id)?;
    }

    Ok(())
}

pub fn validate_persona_deletion(
    persona: &AgentDefinition,
    referenced_by_team: bool,
) -> Result<(), String> {
    if persona.is_builtin {
        return Err("Built-in agents cannot be deleted.".to_string());
    }

    if persona.source_team.is_some() {
        return Err(format!(
            "{} belongs to a team. Delete the team to remove all team agents together.",
            persona.display_name
        ));
    }

    if referenced_by_team {
        return Err(format!(
            "{} is still referenced by a team. Remove it from those teams first.",
            persona.display_name
        ));
    }

    Ok(())
}

pub fn validate_persona_activation_change(
    persona: &AgentDefinition,
    active: bool,
    referenced_by_managed_agent: bool,
    referenced_by_team: bool,
) -> Result<(), String> {
    if !persona.is_builtin {
        return Err("Only built-in agents can be added to or removed from My Agents.".to_string());
    }

    if !active && referenced_by_managed_agent {
        return Err(format!(
            "{} is still assigned to a managed agent. Remove or reassign those agents first.",
            persona.display_name
        ));
    }

    if !active && referenced_by_team {
        return Err(format!(
            "{} is still referenced by a team. Remove it from those teams first.",
            persona.display_name
        ));
    }

    Ok(())
}

pub fn load_personas<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<Vec<AgentDefinition>, String> {
    let now = now_iso();

    // Post-fold: definitions live in the unified agent store, presented in
    // the legacy shape. Pre-fold stores are converted by
    // `fold_personas_into_agent_store` in boot migrations before any caller
    // reaches this shim.
    let records = crate::managed_agents::storage::load_agent_definitions(app)?
        .iter()
        .filter_map(|record| record.to_definition_view())
        .collect();

    let (records, changed) = merge_personas(records, &now);
    if changed {
        save_personas(app, &records)?;
    }

    Ok(records)
}

/// Read the raw persona records at `path` — no built-in merge, no write-back.
/// The single disk-read seam for persona definitions: `load_personas` layers
/// the built-in merge on top, and the boot-time readers that need raw records
/// without an `AppHandle` (`event_sync`, `migration::load_persona_runtimes`)
/// call it directly. The A2 store fold retargets THIS function at the unified
/// store; its callers stay unchanged.
pub(crate) fn load_personas_from_path(
    path: &std::path::Path,
) -> Result<Vec<AgentDefinition>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read persona store: {error}"))?;
    serde_json::from_str::<Vec<AgentDefinition>>(&content)
        .map_err(|error| format!("failed to parse persona store: {error}"))
}

pub fn save_personas<R: tauri::Runtime>(
    app: &AppHandle<R>,
    records: &[AgentDefinition],
) -> Result<(), String> {
    let mut sorted = records.to_vec();
    sort_personas(&mut sorted);

    // Post-fold: persona saves write key-less definition records into the
    // unified agent store (instances preserved by `save_agent_definitions`).
    let definitions: Vec<_> = sorted
        .into_iter()
        .map(|persona| persona.into_agent_record())
        .collect();
    crate::managed_agents::storage::save_agent_definitions(app, &definitions)
}

#[cfg(test)]
mod tests;

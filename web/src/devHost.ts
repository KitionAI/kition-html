/**
 * The development host (dev.html, `pnpm dev`): opens `sample/Sample.htm`, or a file chosen in the
 * page. The sample is `.htm` so Vite's server hands it over as it is (it rewrites `.html` pages).
 */
import { startDevHost } from '@kition/sdk/dev-host'

startDevHost({ sample: './sample/Sample.htm' })

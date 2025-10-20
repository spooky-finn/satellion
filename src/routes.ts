import { type RouteConfig, index, route } from "@react-router/dev/routes"

export default [
  index("routes/_index.tsx"),
  route("create_wallet", "routes/create_wallet.tsx")
] satisfies RouteConfig;
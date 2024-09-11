/// <reference types="vite/client" />

declare module "*.jpg" {
  const value: string;
  export default value;
}
declare module "*.png" {
  const value: string;
  export default value;
}

declare module "*?url" {
  const value: string;
  export default value;
}

declare module "*.glsl" {
  const value: string;
  export default value;
}
declare module "*?raw" {
  const value: string;
  export default value;
}

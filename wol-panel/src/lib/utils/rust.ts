export function unreachable() {
  throw createError({
    statusCode: 404,
    statusMessage: "Unreachable reached",
  });
}

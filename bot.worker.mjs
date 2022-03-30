const Bot = (await import("./Bot")).default;
/**
 * @type {Worker}
 */
const ctx = self;
const bot = new Bot(ctx.postMessage);
/**
 * @param {MessageEvent} event
 */
ctx.onmessage = (event) => {
    bot.onMessage(event.data);
};
bot.init();
export {};
//# sourceMappingURL=bot.worker.mjs.map
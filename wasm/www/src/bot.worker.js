const Bot = (await import("./Bot")).default;

const bot = new Bot(postMessage);
onmessage = (event) => {
  bot.onMessage(event.data);
};
bot.init();

export { };
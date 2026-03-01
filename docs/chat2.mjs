import 'dotenv/config'; // 加载环境变量
import axios from 'axios'; // 使用 axios 发送请求
import readline from 'readline'; // 使用 readline 实现交互式输入
import fs from 'fs'; // 使用 fs 模块保存文件
import path from 'path'; // 使用 path 模块处理文件路径
import chalk from 'chalk'; // 使用 chalk 添加颜色
import { table } from 'table'; // 使用 table 优化排版
import { exec } from 'child_process'; // 用于重启程序

// 默认配置（仅用于初始化外部配置文件）
const defaultConfig = {
  temperature: 0.7,
  maxTokens: 1000,
  enableStream: true,
  timeout: 10000,
  summaryDir: './conversation_summaries',
  truncateLength: 300,
  currentModel: 'deepseek-chat',
  appName: 'DeepSeek',
  apiEndpoint: 'https://api.deepseek.com/v1/chat/completions',
};

// Tab 补全选项
const COMMANDS = [
  '/help',
  '/exit',
  '/save',
  '/load',
  '/update ',
  '/update api',
  '/update web',
  '/set ',
  '/set temperature',
  '/set max_tokens',
  '/set stream true',
  '/set stream false',
  '/set timeout',
  '/set summary_dir',
  '/set truncate_length',
  '/set name',
  '/reset all',
  '/reset temperature',
  '/reset max_tokens',
  '/reset stream',
  '/reset timeout',
  '/reset summary_dir',
  '/reset truncate_length',
  '/reset name',
  '/m v3',
  '/m r1',
  '/del',
  '/config',
  '/upload', // 新增指令
];

// 创建 readline 接口
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

// 监听 Tab 键
rl.input.on('keypress', (char, key) => {
  if (key && key.name === 'tab') {
    // 获取当前输入内容
    const input = rl.line.trim();

    // 查找匹配的补全选项
    const matches = COMMANDS.filter((cmd) => cmd.startsWith(input));

    if (matches.length === 1) {
      // 如果只有一个匹配项，直接补全
      rl.write(null, { ctrl: true, name: 'u' }); // 清空当前行
      rl.write(matches[0] + ' '); // 写入补全内容
    } else if (matches.length > 1) {
      // 如果有多个匹配项，列出所有选项
      console.log('\n可能的补全选项：');
      matches.forEach((match) => console.log(chalk.blue(match)));
      rl.prompt(); // 重新显示提示符
    }
  }
});

// 全局配置变量
let config;

// 加载外部配置文件
function loadConfig() {
  const configFilePath = path.resolve('config.json'); // 配置文件路径

  if (!fs.existsSync(configFilePath)) {
    // 如果配置文件不存在，则创建并写入默认配置
    fs.writeFileSync(configFilePath, JSON.stringify(defaultConfig, null, 2)); // 写入默认配置
    console.log(chalk.green('配置文件已创建并初始化。'));
  }

  // 读取并解析配置文件
  const configData = fs.readFileSync(configFilePath, 'utf-8');
  return JSON.parse(configData);
}

// 保存配置文件
function saveConfig(config) {
  const configFilePath = path.resolve('config.json'); // 配置文件路径
  fs.writeFileSync(configFilePath, JSON.stringify(config, null, 2)); // 写入配置文件
  console.log(chalk.green('配置文件已更新。'));
}

// 在程序启动时加载配置文件
config = loadConfig();

// 更新全局变量
let temperature = config.temperature;
let maxTokens = config.maxTokens;
let enableStream = config.enableStream;
let timeout = config.timeout;
let summaryDir = config.summaryDir;
let truncateLength = config.truncateLength;
let currentModel = config.currentModel;
let appName = config.appName;
let API_ENDPOINT = config.apiEndpoint;

// 设置对话主旨文件的保存目录
if (!fs.existsSync(summaryDir)) {
  fs.mkdirSync(summaryDir); // 如果目录不存在，则创建
}

// 存储对话历史
let conversationHistory = [];

// 检查 API 端点格式
function isValidApiEndpoint(endpoint) {
  const urlPattern = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.-]*)*\/?$/;
  return urlPattern.test(endpoint);
}

// 检查 API 密钥格式
function isValidApiKey(apiKey) {
  return apiKey.length >= 20;
}

// 检查并初始化 .env 文件
function checkAndInitializeEnv() {
  const envFilePath = path.resolve('.env'); // 获取 .env 文件的路径

  if (!fs.existsSync(envFilePath)) {
    rl.question(chalk.yellow('请输入你的 API 密钥：'), (apiKey) => {
      if (!isValidApiKey(apiKey)) {
        console.log(chalk.red('无效的 API 密钥格式，请输入至少 20 位的密钥。'));
        checkAndInitializeEnv(); // 重新提示输入
      } else {
        fs.writeFileSync(envFilePath, `DEEPSEEK_API_KEY=${apiKey}`); // 写入 .env 文件
        console.log(chalk.green('API 密钥已保存到 .env 文件。'));
        restartApp(); // 启动应用程序
      }
    });
  } else {
    startApp(); // 如果 .env 文件存在，直接启动应用程序
  }
}

// 手动更新 API 密钥并重启程序
function updateApiKey() {
  const envFilePath = path.resolve('.env'); // 获取 .env 文件的路径

  rl.question(chalk.yellow('请输入新的 API 密钥：'), (apiKey) => {
    if (apiKey.startsWith('/')) {
      handleCommand(apiKey);
    } else if (!isValidApiKey(apiKey)) {
      console.log(chalk.red('无效的 API 密钥格式，请输入至少 20 位的密钥。'));
      updateApiKey(); // 重新提示输入
    } else {
      fs.writeFileSync(envFilePath, `DEEPSEEK_API_KEY=${apiKey}`); // 更新 .env 文件
      console.log(chalk.green('API 密钥已更新，正在重启程序...'));
      restartApp();
    }
  });
}

// 更新 API 端点
function updateApiEndpoint() {
  rl.question(chalk.yellow('请输入新的 API 端点（例如 https://api.openai.com/v1/chat/completions）：'), (endpoint) => {
    if (endpoint.startsWith('/')) {
      handleCommand(endpoint);
    } else if (!isValidApiEndpoint(endpoint)) {
      console.log(chalk.red('无效的 API 端点格式，请输入有效的 URL。'));
      updateApiEndpoint(); // 重新提示输入
    } else {
      API_ENDPOINT = endpoint; // 更新 API 端点
      console.log(chalk.green(`API 端点已更新为：${API_ENDPOINT}`));
      askQuestion(); // 返回主循环
    }
  });
}

// 更新应用程序名称
function updateAppName() {
  rl.question(chalk.yellow('请输入新的应用程序名称：'), (name) => {
    if (name.startsWith('/')) {
      handleCommand(name);
    } else {
      appName = name; // 更新应用程序名称
      console.log(chalk.green(`应用程序名称已更新为：${appName}`));
      askQuestion(); // 返回主循环
    }
  });
}

// 重启程序
function restartApp() {
  rl.close(); // 关闭 readline 接口
  exec('node ' + process.argv[1], (error, stdout, stderr) => {
    if (error) {
      console.error(chalk.red('重启程序失败:', error.message, '建议手动重启。'));
      return;
    }
    console.log(chalk.green('程序已重启。'));
  });
}

// 清理文件名中的非法字符
function sanitizeFileName(title) {
  return title
    .replace(/[:/\\*?"<>|]/g, '') // 移除非法字符
    .replace(/\s+/g, '_') // 将空格替换为下划线
    .substring(0, 50); // 限制文件名长度
}

// 将 Markdown 语法转换为 ANSI 转义序列
function formatTextWithANSI(text) {
  text = text.replace(/\*\*(.*?)\*\*/g, '\x1b[1m$1\x1b[22m'); // 加粗
  text = text.replace(/_(.*?)_/g, '\x1b[3m$1\x1b[23m'); // 斜体
  text = text.replace(/`(.*?)`/g, '\x1b[32m$1\x1b[0m'); // 高亮
  return text;
}

// 光标动画函数
function startCursorAnimation() {
  const frames = ['-', '\\', '|', '/']; // 旋转动画的帧
  let frameIndex = 0;

  const interval = setInterval(() => {
    process.stdout.write(`\r${frames[frameIndex]} ${appName} 正在思考...`);
    frameIndex = (frameIndex + 1) % frames.length;
  }, 100);

  return interval;
}

// 停止光标动画
function stopCursorAnimation(interval) {
  clearInterval(interval);
  process.stdout.write('\r');
  process.stdout.write(' '.repeat(process.stdout.columns));
  process.stdout.write('\r');
}

// 调用 API 的函数
async function callAPI(messages) {
  const API_KEY = process.env.DEEPSEEK_API_KEY; // 从环境变量读取密钥

  const animationInterval = startCursorAnimation();

  try {
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error('请求超时，请稍后重试。'));
      }, timeout);
    });

    const response = await Promise.race([
      axios.post(
        API_ENDPOINT,
        {
          model: currentModel,
          messages,
          temperature: temperature, // 使用全局变量
          max_tokens: maxTokens, // 使用全局变量
          stream: enableStream, // 使用全局变量
        },
        {
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${API_KEY}`,
          },
          responseType: enableStream ? 'stream' : 'json', // 根据 enableStream 设置 responseType
        }
      ),
      timeoutPromise,
    ]);

    if (enableStream) {
      // 流式输出逻辑
      let fullResponse = '';
      let reasoningProcess = ''; // 存储思考过程
      let isFirstChunk = true; // 标记是否为第一个数据块

      response.data.on('data', (chunk) => {
        const chunkString = chunk.toString();

        if (isFirstChunk) {
          stopCursorAnimation(animationInterval);
          isFirstChunk = false;
        }

        chunkString.split('\n').forEach((line) => {
          if (line.trim() === '') return; // 忽略空行
          if (line.startsWith('data: ')) {
            const jsonString = line.slice(6); // 去掉 "data: " 前缀
            if (jsonString === '[DONE]') {
              console.log('\n流式响应结束。');
              return; // 结束标记，跳过解析
            }

            try {
              const parsedData = JSON.parse(jsonString);
              if (parsedData.choices && parsedData.choices[0] && parsedData.choices[0].delta.content) {
                const content = parsedData.choices[0].delta.content;

                if (content.includes('**思考过程**')) {
                  reasoningProcess += content.replace('**思考过程**', '').trim();
                } else {
                  fullResponse += content;
                }

                const formattedText = formatTextWithANSI(content);
                process.stdout.write(formattedText);
              }
            } catch (error) {
              console.error('解析数据块失败:', error.message);
            }
          }
        });
      });

      await new Promise((resolve, reject) => {
        response.data.on('end', resolve);
        response.data.on('error', reject);
      });

      return { reply: fullResponse, reasoning: reasoningProcess };
    } else {
      // 非流式输出逻辑
      stopCursorAnimation(animationInterval);

      const reply = response.data.choices[0].message.content;
      console.log(chalk.green(`${appName} 的回复：`), formatTextWithANSI(reply));

      return { reply, reasoning: '' };
    }
  } catch (error) {
    stopCursorAnimation(animationInterval);

    if (error.response && error.response.status === 401) {
      console.error(chalk.red('API 密钥无效，请检查你的 API 密钥。'));
      console.log(chalk.yellow('输入 /update api 手动更新 API 密钥。'));
    } else {
      console.error(chalk.red('调用 API 失败:', error.message));
    }
    throw error;
  }
}

// 获取对话主旨信息
async function getConversationSummary() {
  const summaryPrompt = `
  请总结以下对话的主旨信息，并生成一个简短的小标题（不超过 10 个字）。
  回复格式必须严格遵循以下格式：
  标题：<标题内容>
  主旨：<主旨内容>

  对话内容：
  ${conversationHistory.map((msg) => `${msg.role}: ${msg.content}`).join('\n')}
  `;

  const { reply } = await callAPI([{ role: 'user', content: summaryPrompt }]);
  return reply;
}

// 保存对话主旨信息到文件
async function saveSummaryToFile(summary) {
  try {
    const titleMatch = summary.match(/标题：(.*)/);
    const summaryMatch = summary.match(/主旨：(.*)/);

    if (!titleMatch || !summaryMatch) {
      throw new Error('无法提取标题或主旨，请检查对话内容。');
    }

    const title = titleMatch[1].trim();
    const summaryContent = summaryMatch[1].trim();

    const sanitizedTitle = sanitizeFileName(title);
    const fileName = `summary_${Date.now()}_${sanitizedTitle}.txt`;
    const filePath = path.join(summaryDir, fileName);

    if (!filePath) {
      throw new Error('文件路径无效。');
    }

    let fileContent = `标题：${title}\n主旨：${summaryContent}\n\n对话历史：\n`;
    conversationHistory.forEach((msg) => {
      const role = msg.role === 'user' ? '用户' : 'AI';
      const content = msg.content.length > truncateLength ? '[**]' : msg.content; // 使用全局变量
      const timestamp = new Date(msg.timestamp).toLocaleString();
      fileContent += `${timestamp} - ${role}: ${content}\n`;
    });

    fs.writeFileSync(filePath, fileContent);
    console.log(chalk.blue(`对话主旨信息已保存到文件：${filePath}`));
  } catch (error) {
    console.error(chalk.red('保存文件失败:', error.message));
  }
}

// 读取指定主旨文件并加载最新的 10 条对话历史
function loadSummaryFile(index) {
  const files = fs.readdirSync(summaryDir);
  if (index < 1 || index > files.length) {
    console.log(chalk.red('无效的序号。'));
    return null;
  }

  const filePath = path.join(summaryDir, files[index - 1]);
  const content = fs.readFileSync(filePath, 'utf-8');

  const lines = content.split('\n');
  const title = lines[0].replace('标题：', '').trim();
  const summary = lines[1].replace('主旨：', '').trim();
  const history = [];

  let isHistorySection = false;
  for (const line of lines) {
    if (line.startsWith('对话历史：')) {
      isHistorySection = true;
      continue;
    }
    if (isHistorySection && line.trim() !== '') {
      const [timestamp, roleContent] = line.split(' - ').map((s) => s.trim());
      const [role, content] = roleContent.split(':').map((s) => s.trim());
      history.push({
        role: role === '用户' ? 'user' : 'assistant',
        content,
        timestamp: new Date(timestamp).getTime(),
      });
    }
  }

  const latestHistory = history.slice(-10);
  conversationHistory = latestHistory;

  console.log(chalk.blue(`文件内容已加载，标题：${title}，主旨：${summary}`));
  console.log(chalk.yellow(`已加载最新的 ${latestHistory.length} 条对话历史。`));
  return { title, summary, history: latestHistory };
}

// 列出所有主旨文件
function listSummaryFiles() {
  const files = fs.readdirSync(summaryDir);
  if (files.length === 0) {
    console.log(chalk.yellow('没有找到任何主旨文件。'));
    return [];
  }

  const fileList = files.map((file, index) => [index + 1, file]);
  console.log(table([[chalk.bold('序号'), chalk.bold('文件名')], ...fileList]));

  return files;
}

// 删除指定主旨文件
function deleteSummaryFile(index) {
  const files = fs.readdirSync(summaryDir);
  if (index < 1 || index > files.length) {
    console.log(chalk.red('无效的序号。'));
    return false;
  }

  const filePath = path.join(summaryDir, files[index - 1]);
  fs.unlinkSync(filePath);
  console.log(chalk.green(`文件已删除：${filePath}`));
  return true;
}

// 显示指令帮助信息
function showHelp() {
  const commands = [
    [chalk.green('/help'), '显示所有可用指令'],
    [chalk.green('/exit'), '退出程序'],
    [chalk.green('/save'), '保存当前对话主旨'],
    [chalk.green('/load'), '加载历史对话主旨'],
    [chalk.green('/del [index]'), '删除指定主旨文件'],
    [chalk.green('/m [v3/r1]'), '切换模型（v3: deepseek-chat, r1: deepseek-reasoner）'],
    [chalk.green('/config'), '查看配置表'],
    [chalk.green('/update api [new_api_key]'), '更新 API 密钥'],
    [chalk.green('/update web [new_endpoint]'), '更新 API 端点'],
    [chalk.green('/set name [new_name]'), '更新应用程序名称'],
    [chalk.green('/set temperature [value]'), '设置 temperature 参数（0.0 到 2.0）'],
    [chalk.green('/set max_tokens [value]'), '设置 max_tokens 参数（1 到 4096）'],
    [chalk.green('/set stream [true/false]'), '启用或禁用流式输出'],
    [chalk.green('/set timeout [value]'), '设置请求超时时间（毫秒）'],
    [chalk.green('/set summary_dir [path]'), '设置主旨文件存储地址'],
    [chalk.green('/set truncate_length [value]'), '设置超过多少字时用 [**] 代替'],
    [chalk.green('/reset all'), '重置所有配置为默认值'],
    [chalk.green('/reset [param]'), '重置指定配置为默认值'],
    [chalk.green('/upload [file_path]'), '上传文件内容给 AI 处理'], // 新增指令说明
    [chalk.green('/r'), '返回上一级'],
  ];

  const usageInstructions = `
  1. 启动程序后，输入你的 ${chalk.yellow('问题')} 或 ${chalk.yellow('指令')}。
  2. 使用 ${chalk.green('/save')} 保存 ${chalk.blue('当前对话')} 以便下次调用。
  3. 使用 ${chalk.green('/load')} 加载 ${chalk.blue('历史对话')} 来重回话题。
  4. 使用 ${chalk.green('/del [index]')} 删除 ${chalk.red('指定历史对话文件')}。
  5. 使用 ${chalk.green('/m [v3/r1]')} 切换模型。
  6. 使用 ${chalk.green('/config')} 查看 ${chalk.yellow('配置表单')}。
  7. 使用 ${chalk.green('/update api [new_api_key]')} 手动更新 ${chalk.yellow('API 密钥')}。
  8. 使用 ${chalk.green('/update web [new_endpoint]')} 手动更新 ${chalk.yellow('API 端点')}。
  9. 使用 ${chalk.green('/set name [new_name]')} 手动更新 ${chalk.yellow('应用程序名称')}。
  10. 使用 ${chalk.green('/set [param] [value]')} 调整参数。
  11. 使用 ${chalk.green('/reset all')} 重置所有配置为默认值。
  12. 使用 ${chalk.green('/reset [param]')} 重置指定配置为默认值。
  13. 使用 ${chalk.green('/upload [file_path]')} 上传文件内容给 AI 处理。
  14. 使用 ${chalk.green('/exit')} 退出程序。
  `;

  const tableData = [
    [chalk.bold('指令'), chalk.bold('描述')],
    ...commands,
    [chalk.bold('版本号'), '1.6.17'],
    [chalk.bold('制作者'), 'XueChen'],
    [chalk.bold('使用说明'), usageInstructions.trim()],
  ];

  console.log(table(tableData, {
    columns: [
      { alignment: 'left', width: 30 }, // 第一列宽度为 20
      { alignment: 'left', width: 60 }, // 第二列宽度为 60
    ],
  }));
}

// 展示配置文件信息
function showConfig() {
  const config = loadConfig(); // 加载配置文件
  if (!config) return;

  // 将配置信息转换为表格数据
  const tableData = [
    [chalk.bold('配置项'), chalk.bold('当前值')], // 表头
    ...Object.entries(config).map(([key, value]) => [chalk.blue(key), chalk.green(value)]), // 表格内容
  ];

  // 生成表格并输出
  console.log(table(tableData, {
    columns: [
      { alignment: 'left', width: 20 }, // 第一列宽度为 20
      { alignment: 'left', width: 40 }, // 第二列宽度为 40
    ],
  }));
}

// 处理命令输入
async function handleCommand(input) {
  const command = input.slice(1).toLowerCase();
  const args = command.split(' ');

  switch (args[0]) {
    case 'help':
      showHelp();
      askQuestion();
      return;

    case 'exit':
      rl.close();
      return;

    case 'save':
      try {
        console.log(chalk.yellow('正在生成对话主旨信息...'));
        const summary = await getConversationSummary();
        await saveSummaryToFile(summary);

        rl.question(chalk.cyan('是否清除对话历史？（输入 "yes" 确认，其他取消）：'), (confirm) => {
          if (confirm.toLowerCase() === 'yes') {
            conversationHistory = [];
            console.log(chalk.green('对话历史已清除。'));
          }
          askQuestion();
        });
      } catch (error) {
        console.error(chalk.red('生成对话主旨失败:', error.message));
        askQuestion();
      }
      return;

    case 'load':
      const files = listSummaryFiles();
      if (files.length > 0) {
        rl.question(chalk.cyan('请输入要加载的文件序号（输入 /del 序号 删除文件，输入 /r 返回）：'), (indexInput) => {
          if (indexInput.startsWith('/')) {
            handleCommand(indexInput);
          } else {
            const index = Number(indexInput);
            if (isNaN(index) || index < 1 || index > files.length) {
              console.log(chalk.red('无效的序号，请输入有效的文件序号。'));
              askQuestion();
            } else {
              loadSummaryFile(index);
              askQuestion();
            }
          }
        });
      } else {
        askQuestion();
      }
      return;

    case 'r':
      askQuestion();
      return;

    case 'm':
      if (args[1] === 'v3') {
        currentModel = 'deepseek-chat';
        console.log(chalk.green('已切换到 deepseek-chat 模型。'));
      } else if (args[1] === 'r1') {
        currentModel = 'deepseek-reasoner';
        console.log(chalk.green('已切换到 deepseek-reasoner 模型。'));
      } else {
        console.log(chalk.red('未知模型，请输入 /m v3 或 /m r1。'));
      }
      askQuestion();
      return;

    case 'update':
      if (args[1] === 'api') {
        updateApiKey();
      } else if (args[1] === 'web') {
        updateApiEndpoint();
      } else {
        console.log(chalk.red('未知参数，请输入 /help 查看可用指令。'));
        askQuestion();
      }
      return;

    case 'set':
      if (args.length < 3) {
        console.log(chalk.red('缺少参数，请输入 /help 查看指令格式。'));
        askQuestion();
        return;
      }

      const param = args[1];
      const value = args.slice(2).join(' '); // 支持带空格的参数值

      switch (param) {
        case 'temperature':
          const temp = parseFloat(value);
          if (isNaN(temp)) {
            console.log(chalk.red('无效的 temperature 值，请输入一个数字。'));
          } else {
            temperature = temp;
            config.temperature = temp; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`temperature 已设置为：${temperature}`));
          }
          break;

        case 'max_tokens':
          const tokens = parseInt(value, 10);
          if (isNaN(tokens)) {
            console.log(chalk.red('无效的 max_tokens 值，请输入一个整数。'));
          } else {
            maxTokens = tokens;
            config.maxTokens = tokens; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`max_tokens 已设置为：${maxTokens}`));
          }
          break;

        case 'stream':
          if (value === 'true' || value === 'false') {
            enableStream = value === 'true';
            config.enableStream = enableStream; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`stream 已设置为：${enableStream}`));
          } else {
            console.log(chalk.red('无效的 stream 值，请输入 true 或 false。'));
          }
          break;

        case 'timeout':
          const time = parseInt(value, 10);
          if (isNaN(time)) {
            console.log(chalk.red('无效的 timeout 值，请输入一个整数。'));
          } else {
            timeout = time;
            config.timeout = time; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`timeout 已设置为：${timeout} 毫秒`));
          }
          break;

        case 'summary_dir':
          if (!fs.existsSync(value)) {
            console.log(chalk.red('指定的目录不存在，请输入有效的路径。'));
          } else {
            summaryDir = value;
            config.summaryDir = value; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`主旨文件存储地址已设置为：${summaryDir}`));
          }
          break;

        case 'truncate_length':
          const length = parseInt(value, 10);
          if (isNaN(length)) {
            console.log(chalk.red('无效的 truncate_length 值，请输入一个整数。'));
          } else {
            truncateLength = length;
            config.truncateLength = length; // 更新配置文件
            saveConfig(config); // 保存配置文件
            console.log(chalk.green(`超过 ${truncateLength} 字时用 [**] 代替`));
          }
          break;

        case 'name':
          appName = value;
          config.appName = value; // 更新配置文件
          saveConfig(config); // 保存配置文件
          console.log(chalk.green(`应用程序名称已更新为：${appName}`));
          break;

        default:
          console.log(chalk.red('未知参数，请输入 /help 查看可用指令。'));
          break;
      }

      askQuestion();
      return;

    case 'del':
      const index = Number(args[1]);
      if (isNaN(index) || index < 1) {
        console.log(chalk.red('无效的序号，请输入有效的文件序号。'));
      } else {
        deleteSummaryFile(index);
      }
      askQuestion();
      return;

    case 'reset':
      if (args.length === 1 || args[1] === 'all') {
        // 如果只输入 /reset 或 /reset all，则重置所有配置
        Object.assign(config, defaultConfig); // 更新全局配置
        saveConfig(config); // 保存默认配置到文件

        // 更新全局变量
        temperature = config.temperature;
        maxTokens = config.maxTokens;
        enableStream = config.enableStream;
        timeout = config.timeout;
        summaryDir = config.summaryDir;
        truncateLength = config.truncateLength;
        currentModel = config.currentModel;
        appName = config.appName;
        API_ENDPOINT = config.apiEndpoint;

        console.log(chalk.green('所有配置已重置为默认值。'));
      } else {
        // 如果输入 /reset [param]，则重置指定配置
        const param = args[1];
        if (defaultConfig.hasOwnProperty(param)) {
          config[param] = defaultConfig[param]; // 重置指定配置
          saveConfig(config); // 保存更新后的配置

          // 更新全局变量
          switch (param) {
            case 'temperature':
              temperature = config.temperature;
              break;
            case 'maxTokens':
              maxTokens = config.maxTokens;
              break;
            case 'enableStream':
              enableStream = config.enableStream;
              break;
            case 'timeout':
              timeout = config.timeout;
              break;
            case 'summaryDir':
              summaryDir = config.summaryDir;
              break;
            case 'truncateLength':
              truncateLength = config.truncateLength;
              break;
            case 'currentModel':
              currentModel = config.currentModel;
              break;
            case 'appName':
              appName = config.appName;
              break;
            case 'apiEndpoint':
              API_ENDPOINT = config.apiEndpoint;
              break;
          }

          console.log(chalk.green(`配置项 ${param} 已重置为默认值：${config[param]}`));
        } else {
          console.log(chalk.red(`未知配置项：${param}，请输入 /help 查看可用配置项。`));
        }
      }
      askQuestion();
      return;

    case 'config':
      showConfig(); // 展示配置文件信息
      askQuestion();
      return;

    case 'upload':
      if (args.length < 2) {
        console.log(chalk.red('请提供文件路径。'));
        askQuestion();
        return;
      }

      const filePath = args[1];
      if (!fs.existsSync(filePath)) {
        console.log(chalk.red('文件不存在，请检查路径。'));
        askQuestion();
        return;
      }

      try {
        const fileContent = fs.readFileSync(filePath, 'utf-8');
        console.log(chalk.blue(`文件内容已读取：\n${fileContent}`));

        conversationHistory.push({
          role: 'user',
          content: `请分析以下文件内容：\n${fileContent}`,
          timestamp: Date.now(),
        });

        console.log(chalk.yellow(`${appName} 正在分析文件内容...`));
        const { reply } = await callAPI(conversationHistory);

        conversationHistory.push({
          role: 'assistant',
          content: reply,
          timestamp: Date.now(),
        });

        console.log(chalk.green(`${appName} 的回复：`), formatTextWithANSI(reply));
      } catch (error) {
        console.error(chalk.red('读取文件失败:', error.message));
      }

      askQuestion();
      return;

    default:
      console.log(chalk.red('未知指令，请输入 /help 查看可用指令。'));
      askQuestion();
      return;

    
  }
}

// 主函数：实现交互式问答
function askQuestion() {
  rl.question(chalk.cyan('请输入你的问题或指令（输入 /help 查看指令）：'), async (userInput) => {
    // 检查输入是否为空或仅包含空白字符
    if (!userInput || !userInput.trim()) {
      console.log(chalk.red('输入不能为空，请重新输入。'));
      askQuestion(); // 重新提示用户输入
      return;
    }

    if (userInput.startsWith('/')) {
      await handleCommand(userInput);
    } else {
      try {
        conversationHistory.push({
          role: 'user',
          content: userInput,
          timestamp: Date.now(),
        });

        if (conversationHistory.length > 10) {
          conversationHistory = conversationHistory.slice(-10);
        }

        console.log(chalk.yellow(`${appName} 正在思考...`));
        const { reply, reasoning } = await callAPI(conversationHistory);

        conversationHistory.push({
          role: 'assistant',
          content: reply,
          timestamp: Date.now(),
        });

        if (reasoning) {
          console.log(chalk.blue('\n思考过程：'), formatTextWithANSI(reasoning));
        }
      } catch (error) {
        console.error(chalk.red('发生错误:', error.message));
      }

      askQuestion();
    }
  });
}

// 启动应用程序
function startApp() {
  console.log(chalk.green(`${appName} 应用程序已启动。`));
  askQuestion();
}

// 检查并初始化 .env 文件
checkAndInitializeEnv();
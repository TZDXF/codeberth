import { describe, expect, it } from "vitest";
import { parseGitRemote } from "@/lib/git-remote";

// 任务计划中提到的 `normalizeRemoteUrl` 实际导出名为 `parseGitRemote`,
// 实现位于 `src/lib/git-remote.ts`(open-with.ts 不含 URL 归一化逻辑)。
// 测试覆盖:HTTPS / SSH / SCP / 尾部 .git / 端口 / 非法 URL,以及 provider 推断。
describe("parseGitRemote", () => {
  describe("HTTPS 协议", () => {
    it("基本 https 地址", () => {
      expect(parseGitRemote("https://github.com/owner/repo")).toEqual({
        provider: "github",
        url: "https://github.com/owner/repo",
      });
    });

    it("https 端口非默认时保留端口", () => {
      const result = parseGitRemote("https://gitlab.example.com:8443/team/proj");
      expect(result).toEqual({
        provider: "gitlab",
        url: "https://gitlab.example.com:8443/team/proj",
      });
    });

    it("http 自建平台带端口(协议与端口都保留)", () => {
      const result = parseGitRemote("http://192.168.1.3:12580/framework/gis/zc-gis-server.git");
      expect(result).toEqual({
        provider: "generic",
        url: "http://192.168.1.3:12580/framework/gis/zc-gis-server",
      });
    });

    it("无协议头时回退为 https", () => {
      const result = parseGitRemote("github.com/owner/repo");
      expect(result?.provider).toBe("github");
      expect(result?.url).toBe("https://github.com/owner/repo");
    });
  });

  describe("SSH 协议", () => {
    it("ssh:// 风格带 user@", () => {
      const result = parseGitRemote("ssh://git@github.com/owner/repo.git");
      expect(result?.provider).toBe("github");
      expect(result?.url).toBe("https://github.com/owner/repo");
    });

    it("ssh:// 风格自定义端口(端口不在归一化输出中)", () => {
      const result = parseGitRemote("ssh://git@gitlab.example.com:2222/team/proj.git");
      expect(result?.provider).toBe("gitlab");
      expect(result?.url).toBe("https://gitlab.example.com/team/proj");
    });
  });

  describe("SCP 风格", () => {
    it("git@host:owner/repo.git", () => {
      const result = parseGitRemote("git@github.com:owner/repo.git");
      expect(result).toEqual({
        provider: "github",
        url: "https://github.com/owner/repo",
      });
    });

    it("嵌套 group 的 gitlab scp 地址", () => {
      const result = parseGitRemote("git@gitlab.com:group/sub/proj.git");
      expect(result?.provider).toBe("gitlab");
      expect(result?.url).toBe("https://gitlab.com/group/sub/proj");
    });
  });

  describe(".git 后缀与尾斜杠", () => {
    it("去掉尾部 .git", () => {
      expect(parseGitRemote("https://github.com/owner/repo.git")?.url).toBe(
        "https://github.com/owner/repo",
      );
    });

    it("大小写不敏感的 .GIT 后缀", () => {
      expect(parseGitRemote("https://github.com/owner/repo.GIT")?.url).toBe(
        "https://github.com/owner/repo",
      );
    });

    it("去掉尾部多个斜杠", () => {
      expect(parseGitRemote("https://github.com/owner/repo///")?.url).toBe(
        "https://github.com/owner/repo",
      );
    });

    it("scp 风格也去除 .git", () => {
      expect(parseGitRemote("git@github.com:owner/repo.git")?.url).toBe(
        "https://github.com/owner/repo",
      );
    });
  });

  describe("provider 推断", () => {
    it("github", () => {
      expect(parseGitRemote("https://github.com/a/b.git")?.provider).toBe("github");
    });

    it("gitee", () => {
      expect(parseGitRemote("https://gitee.com/a/b.git")?.provider).toBe("gitee");
    });

    it("gitlab", () => {
      expect(parseGitRemote("https://gitlab.com/a/b.git")?.provider).toBe("gitlab");
    });

    it("自建 gitlab(域名包含 gitlab)被识别为 gitlab", () => {
      expect(parseGitRemote("https://gitlab.mycorp.com/a/b.git")?.provider).toBe("gitlab");
    });

    it("未匹配则归为 generic", () => {
      expect(parseGitRemote("https://bitbucket.org/a/b.git")?.provider).toBe("generic");
    });
  });

  describe("非法输入返回 null", () => {
    it("空字符串", () => {
      expect(parseGitRemote("")).toBeNull();
    });

    it("纯空白", () => {
      expect(parseGitRemote("   ")).toBeNull();
    });

    it("URL 解析失败", () => {
      // URL 解析失败且非 scp 风格
      expect(parseGitRemote("not a url with spaces and ::::")).toBeNull();
    });

    it("无 host 也无路径", () => {
      // scp 正则不匹配,URL 解析失败 -> null
      expect(parseGitRemote("://broken")).toBeNull();
    });
  });
});

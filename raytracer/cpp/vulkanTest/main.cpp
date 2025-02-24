#include <vulkan/vulkan.h>

#include <iostream>
#include <fstream>
#include <vector>
#include <cstring>
#include <map>
#include <optional>

#include <stdexcept>
#include <cstdlib>
#include <vulkan/vulkan_core.h>

#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>

class HelloTriangleApplication {
public:
    void run() {
        initWindow();
        initVulkan();
        mainLoop();
        cleanup();
    }

private:
    GLFWwindow* window;
    const uint32_t height = 600;
    const uint32_t width = 800;

    const std::vector<const char*> validationLayers = {
      "VK_LAYER_KHRONOS_validation"
    };

    #ifdef NDEBUG
      const bool enableValidationLayers = false;
    #else
      const bool enableValidationLayers = true;
    #endif

    VkInstance instance;
    VkPhysicalDevice physicalDevice = VK_NULL_HANDLE;

    VkDevice device;

    VkQueue graphicsQueue;

    // Compute pipeline objects
    VkPipeline computePipeline;
    VkPipelineLayout computePipelineLayout;
    VkShaderModule computeShaderModule;
    VkCommandPool computeCommandPool;
    VkCommandBuffer computeCommandBuffer;
    VkQueue computeQueue; // You can use graphicsQueue if that’s all you have.

    // (Optional) Descriptor sets and layouts if you pass data to the shader:
    VkDescriptorSetLayout descriptorSetLayout;
    VkDescriptorPool descriptorPool;
    VkDescriptorSet descriptorSet;


    struct QueueFamilyIndices {
      std::optional<uint32_t> graphicsFamily;
      bool isComplete() {
        return graphicsFamily.has_value();
      }
    };

    
    bool checkValidationLayerSupport() {
      uint32_t layerCount;
      vkEnumerateInstanceLayerProperties(&layerCount, nullptr);
      std::vector<VkLayerProperties> availableLayers(layerCount);
      vkEnumerateInstanceLayerProperties(&layerCount, availableLayers.data());
      for (const char* layerName : validationLayers) {
          bool layerFound = false;

          for (const auto& layerProperties : availableLayers) {
              if (strcmp(layerName, layerProperties.layerName) == 0) {
                  layerFound = true;
                  break;
              }
          }

          if (!layerFound) {
              return false;
          }
      }

      return true;
    } 

    void createInstance() {
        if (enableValidationLayers && !checkValidationLayerSupport()) {
            throw std::runtime_error("validation layers requested, but not available!");
        }
      // Optional: Provide useful info to the driver.
        VkApplicationInfo appInfo{};
        appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
        appInfo.pApplicationName = "Hello Triangle";
        appInfo.applicationVersion = VK_MAKE_VERSION(1, 0, 0);
        appInfo.pEngineName = "No Engine";
        appInfo.engineVersion = VK_MAKE_VERSION(1, 0, 0);
        appInfo.apiVersion = VK_API_VERSION_1_0;

      // Struct to tell the Vulkan driver which extensions and validation layers to use.
        VkInstanceCreateInfo createInfo{};
        createInfo.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        createInfo.pApplicationInfo = &appInfo; // Attach the application info

      // Get the required extensions from GLFW.
        uint32_t glfwExtensionCount = 0;
        const char** glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
        createInfo.enabledExtensionCount = glfwExtensionCount;
        createInfo.ppEnabledExtensionNames = glfwExtensions; // Pass the extensions to Vulkan
        createInfo.enabledLayerCount = 0;

      // Check for available extensions
        uint32_t extensionCount = 0;
        vkEnumerateInstanceExtensionProperties(nullptr, &extensionCount, nullptr);
        std::vector<VkExtensionProperties> extensions(extensionCount);
        vkEnumerateInstanceExtensionProperties(nullptr, &extensionCount, extensions.data());

        std::cout << "available extensions:\n";

        for (const auto& extension : extensions) {
            std::cout << '\t' << extension.extensionName << '\n';
        }

      // Create the Vulkan instance
        VkResult result = vkCreateInstance(&createInfo, nullptr, &instance);
        if (result != VK_SUCCESS) {
            throw std::runtime_error("failed to create instance!");
        } else {
          std::cout << "Vulkan instance created successfully!" << std::endl;
        }
    }

    bool isDeviceSuitable(VkPhysicalDevice device) {
      QueueFamilyIndices indices = findQueueFamilies(device);

      return indices.isComplete();
    }

    int rateDeviceSuitability(VkPhysicalDevice device) {
      VkPhysicalDeviceProperties deviceProperties;
      VkPhysicalDeviceFeatures deviceFeatures;
      vkGetPhysicalDeviceProperties(device, &deviceProperties);
      vkGetPhysicalDeviceFeatures(device, &deviceFeatures);

      int score = 0;
      if (deviceProperties.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) {
        score += 1000;
      }
      
      // Max texture size
      score += deviceProperties.limits.maxImageDimension2D;

      if (!deviceFeatures.geometryShader) {
        return 0;
      }

    std::cout << "Device Score: " << score <<
      "\nDevice name: " << deviceProperties.deviceName << std::endl;

      return score;
    }

    void pickPhysicalDevice() {
      uint32_t deviceCount = 0;
      vkEnumeratePhysicalDevices(instance, &deviceCount, nullptr);
      if (deviceCount == 0) {
        throw std::runtime_error("failed to find GPUs with Vulkan support!");
      }

      std::vector<VkPhysicalDevice> devices(deviceCount);
      vkEnumeratePhysicalDevices(instance, &deviceCount, devices.data());

      std::multimap<int, VkPhysicalDevice> candidates;

    // Check which devices are suitable.
      for (const auto& device : devices) {
        int score = rateDeviceSuitability(device);
        candidates.insert(std::make_pair(score, device));


        // Check if the device is suitable at all
        if (candidates.rbegin()->first > 0) {
            physicalDevice = candidates.rbegin()->second;
        } else {
            throw std::runtime_error("failed to find a suitable GPU!");
        }
      }
    }

    QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device) {
      QueueFamilyIndices indices;
      uint32_t queueFamilyCount = 0;
      vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, nullptr);
      std::vector<VkQueueFamilyProperties> queueFamilies(queueFamilyCount);
      vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, queueFamilies.data());

      int i = 0;
      for (const auto& queueFamily : queueFamilies) {
          if (queueFamily.queueFlags & VK_QUEUE_GRAPHICS_BIT) {
              indices.graphicsFamily = i;
          }

        if (indices.isComplete()) {
          break;
        }

          i++;
      }

      return indices;
    }

    void createLogicalDevice() {
      QueueFamilyIndices indices = findQueueFamilies(physicalDevice);

      VkDeviceQueueCreateInfo queueCreateInfo{};
      queueCreateInfo.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
      queueCreateInfo.queueFamilyIndex = indices.graphicsFamily.value();
      queueCreateInfo.queueCount = 1;

      float queuePriority = 1.0f;
      queueCreateInfo.pQueuePriorities = &queuePriority;

      VkPhysicalDeviceFeatures deviceFeatures{};

      VkDeviceCreateInfo createInfo{};
      createInfo.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;

      // Pointers to the queue create info and the device features
      createInfo.pQueueCreateInfos = &queueCreateInfo;
      createInfo.queueCreateInfoCount = 1;

      createInfo.pEnabledFeatures = &deviceFeatures;
      
      createInfo.enabledExtensionCount = 0;

      if (enableValidationLayers) {
          createInfo.enabledLayerCount = static_cast<uint32_t>(validationLayers.size());
          createInfo.ppEnabledLayerNames = validationLayers.data();
      } else {
          createInfo.enabledLayerCount = 0;
      }

      if (vkCreateDevice(physicalDevice, &createInfo, nullptr, &device) != VK_SUCCESS) {
        throw std::runtime_error("failed to create logical device!");
      }
      vkGetDeviceQueue(device, indices.graphicsFamily.value(), 0, &graphicsQueue);
    }

  std::vector<char> readFile(const std::string& filename) {
    std::ifstream file(filename, std::ios::ate | std::ios::binary);
    if (!file.is_open()) {
        throw std::runtime_error("failed to open file!");
    }
    size_t fileSize = (size_t) file.tellg();
    std::vector<char> buffer(fileSize);
    file.seekg(0);
    file.read(buffer.data(), fileSize);
    file.close();
    return buffer;
}

VkShaderModule createShaderModule(const std::vector<char>& code) {
    VkShaderModuleCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    createInfo.codeSize = code.size();
    createInfo.pCode = reinterpret_cast<const uint32_t*>(code.data());
    
    VkShaderModule shaderModule;
    if (vkCreateShaderModule(device, &createInfo, nullptr, &shaderModule) != VK_SUCCESS) {
        throw std::runtime_error("failed to create shader module!");
    }
    return shaderModule;
}

void createComputePipeline() {
    if (device == VK_NULL_HANDLE) {
        std::cerr << "Error: Logical device is null!" << std::endl;
        return;
    }
    // Load your compute shader SPIR-V code:
    auto shaderCode = readFile("raytracer.spv");
    std::cout << "Shader file size: " << shaderCode.size() << " bytes" << std::endl;
    if (shaderCode.empty()) {
        throw std::runtime_error("Shader file is empty!");
    }
    computeShaderModule = createShaderModule(shaderCode);

    // Create a shader stage info for the compute shader.
    VkPipelineShaderStageCreateInfo shaderStageInfo{};
    shaderStageInfo.sType  = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    shaderStageInfo.stage  = VK_SHADER_STAGE_COMPUTE_BIT;
    shaderStageInfo.module = computeShaderModule;
    shaderStageInfo.pName  = "main"; // entry point

    // (Optional) Create descriptor set layouts if your shader uses them.
    // For simplicity, if your shader only writes to an output image that is bound
    // at binding 0, you might already have declared that in your shader.

    // Create a pipeline layout. (Pass your descriptor set layout if using one.)
    VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
    pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    // pipelineLayoutInfo.setLayoutCount = 1;
    // pipelineLayoutInfo.pSetLayouts = &descriptorSetLayout; // if you have one

    if (vkCreatePipelineLayout(device, &pipelineLayoutInfo, nullptr, &computePipelineLayout) != VK_SUCCESS) {
        throw std::runtime_error("failed to create compute pipeline layout!");
    }

    // Now create the compute pipeline.
    VkComputePipelineCreateInfo pipelineInfo{};
    pipelineInfo.sType  = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    pipelineInfo.stage  = shaderStageInfo;
    pipelineInfo.layout = computePipelineLayout;

    if (vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &computePipeline) != VK_SUCCESS) {
        throw std::runtime_error("failed to create compute pipeline!");
    }
}

void createComputeCommandBuffer() {
    // Create a command pool for compute commands. Use the same queue family index.
    QueueFamilyIndices indices = findQueueFamilies(physicalDevice);  // Already implemented
    VkCommandPoolCreateInfo poolInfo{};
    poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    poolInfo.queueFamilyIndex = indices.graphicsFamily.value();
    poolInfo.flags = 0;
    
    if (vkCreateCommandPool(device, &poolInfo, nullptr, &computeCommandPool) != VK_SUCCESS) {
        throw std::runtime_error("failed to create compute command pool!");
    }

    // Allocate a command buffer.
    VkCommandBufferAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    allocInfo.commandPool = computeCommandPool;
    allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    allocInfo.commandBufferCount = 1;

    if (vkAllocateCommandBuffers(device, &allocInfo, &computeCommandBuffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate compute command buffer!");
    }
}

void recordComputeCommandBuffer() {
    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    beginInfo.flags = 0;
    beginInfo.pInheritanceInfo = nullptr;

    if (vkBeginCommandBuffer(computeCommandBuffer, &beginInfo) != VK_SUCCESS) {
        throw std::runtime_error("failed to begin recording compute command buffer!");
    }

    // Bind the compute pipeline.
    vkCmdBindPipeline(computeCommandBuffer, VK_PIPELINE_BIND_POINT_COMPUTE, computePipeline);

    // Bind descriptor sets if you have any (for output image, etc.)
    // vkCmdBindDescriptorSets(computeCommandBuffer, VK_PIPELINE_BIND_POINT_COMPUTE, computePipelineLayout,
    //                          0, 1, &descriptorSet, 0, nullptr);

    // Calculate dispatch dimensions.
    // For example, if your output image is width x height and your workgroup size is 16x16:
    uint32_t groupCountX = (width + 15) / 16;
    uint32_t groupCountY = (height + 15) / 16;
    vkCmdDispatch(computeCommandBuffer, groupCountX, groupCountY, 1);

    if (vkEndCommandBuffer(computeCommandBuffer) != VK_SUCCESS) {
        throw std::runtime_error("failed to record compute command buffer!");
    }
}

    void initWindow() {
      glfwInit();
      glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
      glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);

      window = glfwCreateWindow(800, 600, "Vulkan", nullptr, nullptr);
    }

    void initVulkan() {
      createInstance();
      //setupDebugMessenger();
      pickPhysicalDevice();
      createLogicalDevice();
      // Create the compute pipeline after the device is ready.
      createComputePipeline();
      createComputeCommandBuffer();
      //recordComputeCommandBuffer();
    }

    void mainLoop() {
        // For a one-off compute dispatch:
        VkSubmitInfo submitInfo{};
        submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
        submitInfo.commandBufferCount = 1;
        submitInfo.pCommandBuffers = &computeCommandBuffer;

        if (vkQueueSubmit(graphicsQueue, 1, &submitInfo, VK_NULL_HANDLE) != VK_SUCCESS) {
            throw std::runtime_error("failed to submit compute command buffer!");
        }
        vkQueueWaitIdle(graphicsQueue);

        // Now you can read back results from your output image/buffer
        // (For testing, you might write these to a file or check them in a debugger)
        while (!glfwWindowShouldClose(window)) {
            glfwPollEvents();
        }
    }

    void cleanup() {
      vkDestroyDevice(device, nullptr);
      vkDestroyInstance(instance, nullptr);
      glfwDestroyWindow(window);
      glfwTerminate();
    }
};

int main() {
    HelloTriangleApplication app;

    try {
        app.run();
    } catch (const std::exception& e) {
        std::cerr << e.what() << std::endl;
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
